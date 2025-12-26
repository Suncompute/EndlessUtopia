/// Main WASM App - manages everything
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window, MouseEvent, KeyboardEvent, WheelEvent, HtmlElement, HtmlInputElement};
use std::rc::Rc;
use std::cell::RefCell;
use crate::world::World;

pub struct App {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    world: World,
    offset_x: f64,
    offset_y: f64,
    cat_x: f64,
    cat_y: f64,
    drawings: Vec<Vec<(f64, f64)>>,
    is_drawing: bool,
    is_panning: bool,
    space_pressed: bool,
    last_x: f64,
    last_y: f64,
    // Terminal
    terminal_x: f64,
    terminal_y: f64,
    terminal_width: f64,
    terminal_height: f64,
    terminal_dragging: bool,
    terminal_drag_offset_x: f64,
    terminal_drag_offset_y: f64,
    terminal_resizing: bool,
    terminal_resize_edge: String,  // "right", "bottom", "corner"
    terminal_input: HtmlInputElement,
    terminal_output: Vec<String>,
    terminal_history: Vec<String>,
    terminal_blink: bool,
    terminal_focused: bool,
}

impl App {
    pub fn new() -> Result<Rc<RefCell<Self>>, JsValue> {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let body = document.body().expect("no body");

        // Create canvas
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        
        canvas.set_width(window.inner_width()?.as_f64().unwrap() as u32);
        canvas.set_height(window.inner_height()?.as_f64().unwrap() as u32);
        
        // Set style via HtmlElement
        let canvas_element: &HtmlElement = canvas.dyn_ref::<HtmlElement>().unwrap();
        let style = canvas_element.style();
        style.set_property("display", "block")?;
        style.set_property("cursor", "crosshair")?;
        
        body.append_child(&canvas)?;

        let ctx = canvas
            .get_context("2d")?
            .expect("no 2d context")
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Terminal input field (off-screen for keyboard input)
        let terminal_input = document
            .create_element("input")?
            .dyn_into::<HtmlInputElement>()?;
        terminal_input.set_type("text");
        terminal_input.set_id("terminal-input");
        terminal_input.set_autocomplete("off");
        terminal_input.set_spellcheck(false);
        let style_term = terminal_input.style();
        style_term.set_property("position", "fixed")?;
        style_term.set_property("opacity", "0")?;
        style_term.set_property("pointer-events", "none")?;
        body.append_child(&terminal_input)?;
        
        // Auto-focus terminal input on page load
        terminal_input.focus().ok();

        // Random cat position
        let cat_x = (js_sys::Math::random() * 4000.0) - 2000.0;
        let cat_y = (js_sys::Math::random() * 4000.0) - 2000.0;

        let canvas_width = window.inner_width()?.as_f64().unwrap();
        let canvas_height = window.inner_height()?.as_f64().unwrap();

        let mut terminal_output = Vec::new();
        terminal_output.push("EndlessUtopia Terminal".to_string());
        terminal_output.push("Type 'help' for commands".to_string());

        let app = Rc::new(RefCell::new(App {
            canvas,
            ctx,
            world: World::new(),
            offset_x: 0.0,
            offset_y: 0.0,
            cat_x,
            cat_y,
            drawings: Vec::new(),
            is_drawing: false,
            is_panning: false,
            space_pressed: false,
            last_x: 0.0,
            last_y: 0.0,
            terminal_x: canvas_width - 280.0 - 8.0,
            terminal_y: canvas_height - 250.0 - 8.0,
            terminal_width: 280.0,
            terminal_height: 250.0,
            terminal_dragging: false,
            terminal_drag_offset_x: 0.0,
            terminal_drag_offset_y: 0.0,
            terminal_resizing: false,
            terminal_resize_edge: String::new(),
            terminal_input: terminal_input.clone(),
            terminal_output,
            terminal_history: Vec::new(),
            terminal_blink: false,
            terminal_focused: false,
        }));

        // Setup event handlers
        app.borrow().setup_events(app.clone(), &window, &terminal_input)?;
        
        // Start render loop
        app.borrow().start_render_loop(app.clone())?;
        
        // Start cat movement
        app.borrow().start_cat_movement(app.clone())?;

        // Start terminal cursor blink
        app.borrow().start_terminal_blink(app.clone())?;

        Ok(app)
    }

    fn start_terminal_blink(&self, app: Rc<RefCell<Self>>) -> Result<(), JsValue> {
        let closure = Closure::wrap(Box::new(move || {
            let mut app = app.borrow_mut();
            app.terminal_blink = !app.terminal_blink;
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                500,
            )?;
        
        closure.forget();
        Ok(())
    }

    fn setup_events(&self, app: Rc<RefCell<Self>>, window: &Window, terminal_input: &HtmlInputElement) -> Result<(), JsValue> {
        // Terminal focus handler
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let mut app = app.borrow_mut();
                app.terminal_focused = true;
            }) as Box<dyn FnMut(_)>);
            
            terminal_input.add_event_listener_with_callback("focus", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Terminal blur handler
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let mut app = app.borrow_mut();
                app.terminal_focused = false;
            }) as Box<dyn FnMut(_)>);
            
            terminal_input.add_event_listener_with_callback("blur", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
        
        // Terminal command handler
        {
            let app = app.clone();
            let terminal_input_clone = terminal_input.clone();
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if event.key() == "Enter" {
                    let command = terminal_input_clone.value().trim().to_string();
                    if !command.is_empty() {
                        let mut app = app.borrow_mut();
                        app.execute_command(&command);
                        terminal_input_clone.set_value("");
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            terminal_input.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Mouse down
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let mut app = app.borrow_mut();
                app.on_mouse_down(event);
            }) as Box<dyn FnMut(_)>);
            
            self.canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Mouse move
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let mut app = app.borrow_mut();
                app.on_mouse_move(event);
            }) as Box<dyn FnMut(_)>);
            
            self.canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Mouse up
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
                let mut app = app.borrow_mut();
                app.on_mouse_up();
            }) as Box<dyn FnMut(_)>);
            
            self.canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Context menu
        {
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                event.prevent_default();
            }) as Box<dyn FnMut(_)>);
            
            self.canvas.add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Wheel
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |event: WheelEvent| {
                event.prevent_default();
                let mut app = app.borrow_mut();
                app.offset_x += event.delta_x() * 0.5;
                app.offset_y += event.delta_y() * 0.5;
            }) as Box<dyn FnMut(_)>);
            
            self.canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Keyboard down
        {
            let app = app.clone();
            let document = window.document().expect("no document");
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                let mut app = app.borrow_mut();
                app.on_key_down(event);
            }) as Box<dyn FnMut(_)>);
            
            document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Keyboard up
        {
            let app = app.clone();
            let document = window.document().expect("no document");
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                let mut app = app.borrow_mut();
                app.on_key_up(event);
            }) as Box<dyn FnMut(_)>);
            
            document.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // Window resize
        {
            let app = app.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let app = app.borrow();
                if let Some(window) = web_sys::window() {
                    if let (Ok(w), Ok(h)) = (window.inner_width(), window.inner_height()) {
                        app.canvas.set_width(w.as_f64().unwrap() as u32);
                        app.canvas.set_height(h.as_f64().unwrap() as u32);
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    fn set_cursor(&self, cursor: &str) {
        if let Some(element) = self.canvas.dyn_ref::<HtmlElement>() {
            element.style().set_property("cursor", cursor).ok();
        }
    }

    fn on_mouse_down(&mut self, event: MouseEvent) {
        let button = event.button();
        let mx = event.client_x() as f64;
        let my = event.client_y() as f64;
        
        // Check if clicking inside terminal area
        if button == 0 &&
           mx >= self.terminal_x && mx <= self.terminal_x + self.terminal_width &&
           my >= self.terminal_y && my <= self.terminal_y + self.terminal_height {
            
            web_sys::console::log_1(&format!("Click in terminal: mx={}, my={}, tx={}, ty={}", mx, my, self.terminal_x, self.terminal_y).into());
            
            let edge_threshold = 10.0;  // Larger threshold for easier resizing
            let tx = self.terminal_x;
            let ty = self.terminal_y;
            let tw = self.terminal_width;
            let th = self.terminal_height;
            
            // Check for resize zones (edges and corner)
            let at_right_edge = mx >= tx + tw - edge_threshold;
            let at_bottom_edge = my >= ty + th - edge_threshold;
            
            web_sys::console::log_1(&format!("at_right_edge={}, at_bottom_edge={}, header_check={}", at_right_edge, at_bottom_edge, my <= ty + 20.0).into());
            
            // Corner resize (bottom-right) - highest priority
            if at_right_edge && at_bottom_edge {
                web_sys::console::log_1(&"CORNER RESIZE".into());
                self.terminal_resizing = true;
                self.terminal_resize_edge = "corner".to_string();
                self.set_cursor("nwse-resize");
                return;
            }
            // Right edge resize
            else if at_right_edge {
                web_sys::console::log_1(&"RIGHT RESIZE".into());
                self.terminal_resizing = true;
                self.terminal_resize_edge = "right".to_string();
                self.set_cursor("ew-resize");
                return;
            }
            // Bottom edge resize
            else if at_bottom_edge {
                web_sys::console::log_1(&"BOTTOM RESIZE".into());
                self.terminal_resizing = true;
                self.terminal_resize_edge = "bottom".to_string();
                self.set_cursor("ns-resize");
                return;
            }
            // Header dragging (header is 20px high)
            else if my <= ty + 20.0 {
                web_sys::console::log_1(&"HEADER DRAG".into());
                self.terminal_dragging = true;
                self.terminal_drag_offset_x = mx - tx;
                self.terminal_drag_offset_y = my - ty;
                self.set_cursor("move");
                return;
            }
            // Only focus if not in header or resize zones
            else {
                web_sys::console::log_1(&"FOCUS".into());
                // Schedule focus for next event loop tick to avoid borrow conflict
                let input_clone = self.terminal_input.clone();
                let _ = web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        wasm_bindgen::closure::Closure::once_into_js(move || {
                            input_clone.focus().ok();
                        }).as_ref().unchecked_ref(),
                        0
                    );
                return;
            }
        }
        
        if button == 1 || button == 2 || (button == 0 && self.space_pressed) {
            // Panning
            self.is_panning = true;
            self.last_x = mx;
            self.last_y = my;
            self.set_cursor("grabbing");
        } else if button == 0 {
            // Drawing
            self.is_drawing = true;
            let world_pos = self.screen_to_world(mx, my);
            self.drawings.push(vec![world_pos]);
        }
    }

    fn on_mouse_move(&mut self, event: MouseEvent) {
        let mx = event.client_x() as f64;
        let my = event.client_y() as f64;

        if self.terminal_dragging {
            self.terminal_x = mx - self.terminal_drag_offset_x;
            self.terminal_y = my - self.terminal_drag_offset_y;
            return;
        }
        
        // Terminal resizing
        if self.terminal_resizing {
            let min_width = 200.0;
            let min_height = 150.0;
            
            match self.terminal_resize_edge.as_str() {
                "right" => {
                    let new_width = (mx - self.terminal_x).max(min_width);
                    self.terminal_width = new_width;
                }
                "bottom" => {
                    let new_height = (my - self.terminal_y).max(min_height);
                    self.terminal_height = new_height;
                }
                "corner" => {
                    let new_width = (mx - self.terminal_x).max(min_width);
                    let new_height = (my - self.terminal_y).max(min_height);
                    self.terminal_width = new_width;
                    self.terminal_height = new_height;
                }
                _ => {}
            }
            return;
        }

        if !self.is_panning && !self.is_drawing {
            let edge_threshold = 8.0;
            let tx = self.terminal_x;
            let ty = self.terminal_y;
            let tw = self.terminal_width;
            let th = self.terminal_height;
            
            // Check if hovering over terminal or resize zones
            if mx >= tx && mx <= tx + tw && my >= ty && my <= ty + th {
                let at_right_edge = mx >= tx + tw - edge_threshold;
                let at_bottom_edge = my >= ty + th - edge_threshold;
                let at_header = my <= ty + 20.0;
                
                if at_right_edge && at_bottom_edge {
                    self.set_cursor("nwse-resize");
                } else if at_right_edge {
                    self.set_cursor("ew-resize");
                } else if at_bottom_edge {
                    self.set_cursor("ns-resize");
                } else if at_header {
                    self.set_cursor("move");
                } else {
                    self.set_cursor("text");
                }
            } else {
                let cursor = if self.space_pressed { "grab" } else { "crosshair" };
                self.set_cursor(cursor);
            }
        }

        if self.is_panning {
            let dx = mx - self.last_x;
            let dy = my - self.last_y;
            self.offset_x -= dx;
            self.offset_y -= dy;
            self.last_x = mx;
            self.last_y = my;
        } else if self.is_drawing {
            let world_pos = self.screen_to_world(mx, my);
            if let Some(last_drawing) = self.drawings.last_mut() {
                last_drawing.push(world_pos);
            }
        }
    }

    fn on_mouse_up(&mut self) {
        self.terminal_dragging = false;
        self.terminal_resizing = false;
        self.is_panning = false;
        self.is_drawing = false;
        // Note: cursor will be updated by next mouse_move event
    }

    fn on_key_down(&mut self, event: KeyboardEvent) {
        // Ignore all navigation keys if terminal is focused
        if self.terminal_focused {
            return;
        }
        
        let code = event.code();
        let key = event.key();

        if code == "Space" && !self.space_pressed {
            self.space_pressed = true;
            self.set_cursor("grab");
            event.prevent_default();
            return;
        }

        let speed = if event.shift_key() { 100.0 } else { 20.0 };

        match key.as_str() {
            "ArrowLeft" => self.offset_x -= speed,
            "ArrowRight" => self.offset_x += speed,
            "ArrowUp" => self.offset_y -= speed,
            "ArrowDown" => self.offset_y += speed,
            _ => {}
        }
    }

    fn on_key_up(&mut self, event: KeyboardEvent) {
        if self.terminal_focused {
            return;
        }
        
        if event.code() == "Space" {
            self.space_pressed = false;
            self.set_cursor("crosshair");
        }
    }

    fn start_render_loop(&self, app: Rc<RefCell<Self>>) -> Result<(), JsValue> {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            app.borrow_mut().render().ok();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
        Ok(())
    }

    fn start_cat_movement(&self, app: Rc<RefCell<Self>>) -> Result<(), JsValue> {
        let closure = Closure::wrap(Box::new(move || {
            let mut app = app.borrow_mut();
            app.cat_x += (js_sys::Math::random() - 0.5) * 40.0;
            app.cat_y += (js_sys::Math::random() - 0.5) * 40.0;
            
            if js_sys::Math::random() < 0.1 {
                app.cat_x += (js_sys::Math::random() - 0.5) * 200.0;
                app.cat_y += (js_sys::Math::random() - 0.5) * 200.0;
            }
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                2000,
            )?;
        
        closure.forget();
        Ok(())
    }

    fn render(&mut self) -> Result<(), JsValue> {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear
        self.ctx.set_fill_style_str("#111");
        self.ctx.fill_rect(0.0, 0.0, width, height);

        // World
        self.render_world(width, height)?;

        // Grid
        self.render_grid(width, height)?;

        // Drawings
        self.render_drawings(width, height)?;

        // Cat
        self.render_cat(width, height)?;

        // UI
        self.render_ui(width, height)?;

        Ok(())
    }

    fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;
        (
            screen_x + self.offset_x - width / 2.0,
            screen_y + self.offset_y - height / 2.0,
        )
    }

    fn world_to_screen(&self, world_x: f64, world_y: f64, width: f64, height: f64) -> (f64, f64) {
        (
            world_x - self.offset_x + width / 2.0,
            world_y - self.offset_y + height / 2.0,
        )
    }

    fn render_world(&mut self, width: f64, height: f64) -> Result<(), JsValue> {
        let char_width = 7.2;
        let char_height = 16.0;
        
        let view_left = self.offset_x - width / 2.0;
        let view_top = self.offset_y - height / 2.0;
        
        // Safe conversion with bounds checking
        let world_col_start = (view_left / char_width).floor();
        let world_row_start = (view_top / char_height).floor();
        
        // Clamp to safe i32 range to prevent overflow
        let world_col_start = world_col_start.clamp(i32::MIN as f64, i32::MAX as f64) as i32 - 20;
        let world_row_start = world_row_start.clamp(i32::MIN as f64, i32::MAX as f64) as i32 - 20;
        
        // Cap rendering area to reasonable size (viewport + buffer)
        let cols = ((width / char_width).ceil() as usize + 60).min(500);
        let rows = ((height / char_height).ceil() as usize + 50).min(300);

        let region_text = self.world.render_region(world_col_start, world_row_start, cols, rows);
        let lines: Vec<&str> = region_text.lines().collect();

        self.ctx.set_fill_style_str("#0f0");
        self.ctx.set_font("12px monospace");
        self.ctx.set_global_alpha(0.4);

        for (row_idx, line) in lines.iter().enumerate() {
            let world_x = world_col_start as f64 * char_width;
            let world_y = (world_row_start + row_idx as i32) as f64 * char_height;
            let (sx, sy) = self.world_to_screen(world_x, world_y, width, height);
            self.ctx.fill_text(line, sx, sy)?;
        }

        self.ctx.set_global_alpha(1.0);
        Ok(())
    }

    fn render_grid(&self, width: f64, height: f64) -> Result<(), JsValue> {
        self.ctx.set_stroke_style_str("#1a1a1a");
        self.ctx.set_line_width(1.0);

        let grid_size = 100.0;
        let start_x = ((self.offset_x - width / 2.0) / grid_size).floor() * grid_size;
        let end_x = ((self.offset_x + width / 2.0) / grid_size).ceil() * grid_size;
        let start_y = ((self.offset_y - height / 2.0) / grid_size).floor() * grid_size;
        let end_y = ((self.offset_y + height / 2.0) / grid_size).ceil() * grid_size;

        let mut gx = start_x;
        while gx <= end_x {
            let (sx, sy1) = self.world_to_screen(gx, start_y, width, height);
            let (_, sy2) = self.world_to_screen(gx, end_y, width, height);
            self.ctx.begin_path();
            self.ctx.move_to(sx, sy1);
            self.ctx.line_to(sx, sy2);
            self.ctx.stroke();
            gx += grid_size;
        }

        let mut gy = start_y;
        while gy <= end_y {
            let (sx1, sy) = self.world_to_screen(start_x, gy, width, height);
            let (sx2, _) = self.world_to_screen(end_x, gy, width, height);
            self.ctx.begin_path();
            self.ctx.move_to(sx1, sy);
            self.ctx.line_to(sx2, sy);
            self.ctx.stroke();
            gy += grid_size;
        }

        Ok(())
    }

    fn render_drawings(&self, width: f64, height: f64) -> Result<(), JsValue> {
        self.ctx.set_stroke_style_str("#00ff00");
        self.ctx.set_line_width(3.0);
        self.ctx.set_line_cap("round");

        for drawing in &self.drawings {
            if drawing.is_empty() {
                continue;
            }

            self.ctx.begin_path();
            for (i, &(wx, wy)) in drawing.iter().enumerate() {
                let (sx, sy) = self.world_to_screen(wx, wy, width, height);
                if i == 0 {
                    self.ctx.move_to(sx, sy);
                } else {
                    self.ctx.line_to(sx, sy);
                }
            }
            self.ctx.stroke();
        }

        Ok(())
    }

    fn render_cat(&self, width: f64, height: f64) -> Result<(), JsValue> {
        let cat_art = [" /\\_/\\  ", "( o.o ) ", " > ^ <  "];

        self.ctx.save();
        self.ctx.set_fill_style_str("#ff0");
        self.ctx.set_font("bold 16px monospace");
        self.ctx.set_shadow_blur(10.0);
        self.ctx.set_shadow_color("#ff0");

        for (i, line) in cat_art.iter().enumerate() {
            let (sx, sy) = self.world_to_screen(self.cat_x, self.cat_y + i as f64 * 20.0, width, height);
            self.ctx.fill_text(line, sx, sy)?;
        }

        self.ctx.restore();
        Ok(())
    }

    fn render_ui(&self, width: f64, height: f64) -> Result<(), JsValue> {
        // Logo oben links
        self.ctx.save();
        
        self.ctx.set_fill_style_str("#0f0");
        self.ctx.set_shadow_blur(5.0);
        self.ctx.set_shadow_color("#0f0");
        self.ctx.set_font("bold 11px monospace");
        
        let logo = vec![
            " ___           _ _               ",
            "| __|_ _  __| | |___ ________",
            "| _|| ' \\/ _` | / -_|_-<_-<",
            "|___|_||_\\__,_|_\\___/__/__/",
            "   _   _  _              _      ",
            "  | | | || |_ ___ _ __(_)__ _ ",
            "  | |_| ||  _/ _ \\ '_ \\ / _` |",
            "   \\___/ \\__\\___/ .__/_\\__,_|",
            "                |_|           ",
        ];
        
        let mut y = 25.0;
        for line in &logo {
            self.ctx.fill_text(line, 15.0, y)?;
            y += 13.0;
        }
        
        self.ctx.restore();
        
        // Bottom left: Position display
        self.ctx.save();
        
        self.ctx.set_fill_style_str("#0f0");
        self.ctx.set_shadow_blur(2.0);
        self.ctx.set_shadow_color("#0f0");
        self.ctx.set_font("bold 11px monospace");
        
        let base_y = height - 12.0;
        
        // Current position
        let pos_text = format!("Position: X: {:.0}  Y: {:.0}", self.offset_x, self.offset_y);
        self.ctx.fill_text(&pos_text, 10.0, base_y)?;
        
        self.ctx.restore();
        
        // Terminal (draggable)
        self.render_terminal(width, height)?;
        
        Ok(())
    }

    fn render_terminal(&self, _width: f64, _height: f64) -> Result<(), JsValue> {
        self.ctx.save();
        
        let tx = self.terminal_x;
        let ty = self.terminal_y;
        let tw = self.terminal_width;
        let th = self.terminal_height;
        
        // Terminal background - pure black like PuTTY
        self.ctx.set_fill_style_str("rgba(0, 0, 0, 0.98)");
        self.ctx.fill_rect(tx, ty, tw, th);
        
        // Terminal border - subtle gray like PuTTY window
        self.ctx.set_stroke_style_str("#444444");
        self.ctx.set_line_width(1.0);
        self.ctx.stroke_rect(tx, ty, tw, th);
        
        // Terminal header (draggable area) - dark gray title bar
        self.ctx.set_fill_style_str("rgba(32, 32, 32, 0.95)");
        self.ctx.fill_rect(tx, ty, tw, 20.0);
        
        // Header text - PuTTY style hostname
        self.ctx.set_fill_style_str("#bbbbbb");
        self.ctx.set_font("11px 'Courier New', monospace");
        self.ctx.fill_text("explorer@endlessutopia:~", tx + 8.0, ty + 14.0)?;
        
        // History area starts right after header
        self.ctx.set_fill_style_str("#ffaa00");
        self.ctx.set_font("13px 'Courier New', monospace");
        
        let history_start_y = ty + 35.0;
        let max_width = tw - 20.0;
        let char_width = 7.8;
        let max_chars = (max_width / char_width).floor() as usize;
        let line_height = 16.0;
        let prompt_height = 20.0;
        // let available_height = th - (history_start_y - ty) - prompt_height - 10.0;
        
        // Process lines with wrapping
        let mut wrapped_lines: Vec<String> = Vec::new();
        for line in &self.terminal_output {
            if line.len() > max_chars {
                let mut remaining = line.as_str();
                while !remaining.is_empty() {
                    let chunk_end = if remaining.len() > max_chars {
                        max_chars
                    } else {
                        remaining.len()
                    };
                    wrapped_lines.push(remaining[..chunk_end].to_string());
                    remaining = &remaining[chunk_end..];
                }
            } else {
                wrapped_lines.push(line.clone());
            }
        }
        
        // --- NEU: Prompt und History teilen sich den Platz ---
        let prompt = "explorer@endlessutopia:~$ ";
        let cursor = if self.terminal_blink { "█" } else { " " };
        let current_input = self.terminal_input.value();
        let prompt_and_input = format!("{}{}{}", prompt, current_input, cursor);
        let max_chars = (max_width / char_width).floor() as usize;
        // UTF-8 safe wrapping using char_indices
        let mut prompt_lines = Vec::new();
        let mut start = 0;
        let mut char_count = 0;
        for (idx, _) in prompt_and_input.char_indices() {
            if char_count > 0 && char_count % max_chars == 0 {
                prompt_lines.push(prompt_and_input[start..idx].to_string());
                start = idx;
            }
            char_count += 1;
        }
        if start < prompt_and_input.len() {
            prompt_lines.push(prompt_and_input[start..].to_string());
        }
        let prompt_lines_len = prompt_lines.len();
        // Wie viele Zeilen passen insgesamt?
        let total_lines = ((th - (history_start_y - ty) - 10.0) / line_height).floor() as usize;
        // Wenn Prompt mehrzeilig ist, bleibt er immer direkt unter der letzten History-Zeile
        let visible_history_lines = if total_lines > prompt_lines_len {
            total_lines - prompt_lines_len
        } else {
            0
        };
        let start_idx = if wrapped_lines.len() > visible_history_lines {
            wrapped_lines.len() - visible_history_lines
        } else {
            0
        };
        // History zeichnen
        let mut line_y = history_start_y;
        for line in &wrapped_lines[start_idx..] {
            if line_y + line_height > ty + th - 10.0 - (prompt_lines_len as f64 * line_height) {
                break;
            }
            self.ctx.fill_text(line, tx + 10.0, line_y)?;
            line_y += line_height;
        }
        // Prompt immer direkt nach der letzten History-Zeile
        let mut y = line_y;
        for line in &prompt_lines {
            if y + 15.0 <= ty + th {
                self.ctx.fill_text(line, tx + 10.0, y)?;
            }
            y += line_height;
        }
        
        self.ctx.restore();
        
        Ok(())
    }

    fn execute_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        self.terminal_history.push(cmd.to_string());
        
        // Show the command that was entered (with prompt)
        self.terminal_output.push(format!("explorer@endlessutopia:~$ {}", cmd));
        
        if parts.is_empty() {
            return;
        }

        match parts[0].to_lowercase().as_str() {
            "help" => {
                self.terminal_output.push("Commands:".to_string());
                self.terminal_output.push("  help        show this help".to_string());
                self.terminal_output.push("  clear       clear terminal".to_string());
                self.terminal_output.push("  goto X Y    jump to coordinates".to_string());
                self.terminal_output.push("  cat         find the ascii cat".to_string());
                self.terminal_output.push("  random      random location".to_string());
                self.terminal_output.push("  pos         show current position".to_string());
                self.terminal_output.push("".to_string());
            }
            "clear" | "cls" => {
                self.terminal_output.clear();
            }
            "goto" => {
                if parts.len() >= 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                        if x.is_finite() && y.is_finite() {
                            self.offset_x = x;
                            self.offset_y = y;
                            self.terminal_output.push(format!("teleported to ({}, {})", x, y));
                        } else {
                            self.terminal_output.push("error: invalid coordinates".to_string());
                        }
                    } else {
                        self.terminal_output.push("error: invalid numbers".to_string());
                    }
                } else {
                    self.terminal_output.push("usage: goto <x> <y>".to_string());
                }
            }
            "cat" => {
                self.offset_x = self.cat_x;
                self.offset_y = self.cat_y;
                self.terminal_output.push("found ascii cat! 🐱".to_string());
            }
            "random" | "rnd" => {
                self.offset_x = (js_sys::Math::random() * 4000.0) - 2000.0;
                self.offset_y = (js_sys::Math::random() * 4000.0) - 2000.0;
                self.terminal_output.push(format!("warped to ({:.0}, {:.0})", self.offset_x, self.offset_y));
            }
            "pos" | "position" | "where" => {
                self.terminal_output.push(format!("x={:.0} y={:.0}", self.offset_x, self.offset_y));
            }
            "" => {
                // Empty command, do nothing
            }
            _ => {
                self.terminal_output.push(format!("command not found: {}", parts[0]));
                self.terminal_output.push("type 'help' for available commands".to_string());
            }
        }
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
