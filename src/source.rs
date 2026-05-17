#![allow(dead_code)]

use crate::{
    i18n::I18n,
    plugins::{AppHook, AppHost},
    tools::{CursorTool, Tool, brush::Brush, shape::Shape},
    ui::panel::{Panel, app::AppPanel},
};

#[allow(dead_code)]
pub struct SourceManager {
    language_package: Vec<Box<dyn I18n>>,
    brush_package: Vec<Box<dyn Brush>>,
    shape_package: Vec<Box<dyn Shape>>,
    tool_package: Vec<Box<dyn Tool>>,
    cursor_tool_package: Vec<Box<dyn CursorTool>>,
    panel_package: Vec<Box<dyn Panel>>,
    app_panel_package: Vec<Box<dyn AppPanel>>,
    hook_package: Vec<Box<dyn AppHook>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            language_package: Vec::new(),
            brush_package: Vec::new(),
            shape_package: Vec::new(),
            tool_package: Vec::new(),
            cursor_tool_package: Vec::new(),
            panel_package: Vec::new(),
            app_panel_package: Vec::new(),
            hook_package: Vec::new(),
        }
    }

    pub fn load_language(&mut self, language: Box<dyn I18n>) {
        self.language_package.push(language);
    }

    pub fn load_tool(&mut self, tool: Box<dyn Tool>) {
        self.tool_package.push(tool);
    }

    pub fn load_cursor_tool(&mut self, tool: Box<dyn CursorTool>) {
        self.cursor_tool_package.push(tool);
    }

    pub fn load_brush(&mut self, brush: Box<dyn Brush>) {
        self.brush_package.push(brush);
    }

    pub fn load_shape(&mut self, shape: Box<dyn Shape>) {
        self.shape_package.push(shape);
    }

    pub fn load_panel(&mut self, panel: Box<dyn Panel>) {
        self.panel_package.push(panel);
    }

    pub fn load_app_panel(&mut self, panel: Box<dyn AppPanel>) {
        self.app_panel_package.push(panel);
    }

    pub fn load_hook(&mut self, hook: Box<dyn AppHook>) {
        self.hook_package.push(hook);
    }

    pub fn load_into(&mut self, app_host: &mut dyn AppHost) {
        for tool in self.tool_package.drain(..) {
            app_host.load_tool(tool);
        }
        for tool in self.cursor_tool_package.drain(..) {
            app_host.load_cursor_tool(tool);
        }
        for brush in self.brush_package.drain(..) {
            app_host.load_brush(brush);
        }
        for shape in self.shape_package.drain(..) {
            app_host.load_shape(shape);
        }
        for panel in self.panel_package.drain(..) {
            app_host.load_panel(panel);
        }
        for panel in self.app_panel_package.drain(..) {
            app_host.load_app_panel(panel);
        }
        for hook in self.hook_package.drain(..) {
            app_host.load_hook(hook);
        }
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}
