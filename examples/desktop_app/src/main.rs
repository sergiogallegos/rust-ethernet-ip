use eframe::egui;
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};
use std::sync::Arc;
use tokio::sync::Mutex;

struct DesktopApp {
    // Connection
    plc_address: String,
    cpu_slot: u8,
    use_route_path: bool,
    connected: bool,
    connection_status: String,

    // Client
    client: Option<Arc<Mutex<EipClient>>>,

    // Tag operations
    tag_name: String,
    tag_value: String,
    tag_result: String,
    tag_type: TagType,

    // Array operations
    array_name: String,
    array_index: i32,
    array_value: String,
    array_result: String,

    // UDT operations
    udt_name: String,
    udt_result: String,
    udt_member_path: String,
    udt_member_result: String,

    // Tab selection
    selected_tab: usize,

    // Log
    log_messages: Vec<String>,

    // Runtime handle
    rt: tokio::runtime::Runtime,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self {
            plc_address: String::new(),
            cpu_slot: 0,
            use_route_path: false,
            connected: false,
            connection_status: "Disconnected".to_string(),
            client: None,
            tag_name: String::new(),
            tag_value: String::new(),
            tag_result: String::new(),
            tag_type: TagType::Dint,
            array_name: String::new(),
            array_index: 0,
            array_value: String::new(),
            array_result: String::new(),
            udt_name: String::new(),
            udt_result: String::new(),
            udt_member_path: String::new(),
            udt_member_result: String::new(),
            selected_tab: 0,
            log_messages: Vec::new(),
            rt: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum TagType {
    #[default]
    Dint,
    Real,
    Bool,
    Int,
    String,
    Udt,
}

impl DesktopApp {
    fn add_log(&mut self, message: String) {
        self.log_messages.push(format!(
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            message
        ));
        // Keep only last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }

    fn connect(&mut self) {
        if self.connected {
            self.add_log("Already connected".to_string());
            return;
        }

        self.add_log(format!("Connecting to {}...", self.plc_address));
        self.connection_status = "Connecting...".to_string();

        let address = if self.plc_address.contains(':') {
            self.plc_address.clone()
        } else {
            format!("{}:44818", self.plc_address)
        };

        let use_route = self.use_route_path;
        let cpu_slot = self.cpu_slot;

        let client_result = self.rt.block_on(async {
            if use_route {
                let route_path = RoutePath::new().add_backplane(1, cpu_slot);
                EipClient::with_route_path(&address, route_path).await
            } else {
                EipClient::connect(&address).await
            }
        });

        match client_result {
            Ok(client) => {
                self.client = Some(Arc::new(Mutex::new(client)));
                self.connected = true;
                self.connection_status = "Connected".to_string();
                self.add_log("✅ Connected successfully".to_string());
            }
            Err(e) => {
                self.connection_status = format!("Error: {}", e);
                self.add_log(format!("❌ Connection failed: {}", e));
            }
        }
    }

    fn disconnect(&mut self) {
        if let Some(_client) = &self.client {
            // Disconnect is handled by drop
            self.client = None;
            self.connected = false;
            self.connection_status = "Disconnected".to_string();
            self.add_log("Disconnected".to_string());
        }
    }

    fn read_tag(&mut self) {
        if !self.connected {
            self.tag_result = "Not connected".to_string();
            self.add_log("Not connected to PLC".to_string());
            return;
        }

        if self.tag_name.is_empty() {
            self.tag_result = "Please enter a tag name".to_string();
            self.add_log("Tag name is empty".to_string());
            return;
        }

        let client = self.client.as_ref().unwrap().clone();
        let tag_name = self.tag_name.clone();

        self.add_log(format!("Reading tag: {}", tag_name));
        self.tag_result = "Reading...".to_string();

        let result = self.rt.block_on(async {
            let mut client_guard = client.lock().await;
            let result = client_guard.read_tag(&tag_name).await;
            drop(client_guard); // Release lock before returning
            result
        });

        match result {
            Ok(value) => {
                let result_str = format!("✅ {:?}", value);
                self.tag_result = result_str.clone();
                self.add_log(format!("✅ Read {}: {:?}", tag_name, value));
                eprintln!("[DESKTOP_APP] Successfully read {}: {:?}", tag_name, value);
            }
            Err(e) => {
                let error_str = format!("❌ Error: {}", e);
                self.tag_result = error_str.clone();
                self.add_log(format!("❌ Failed to read {}: {}", tag_name, e));
                eprintln!("[DESKTOP_APP] Error reading {}: {}", tag_name, e);
            }
        }
    }

    fn write_tag(&mut self) {
        if !self.connected {
            self.tag_result = "Not connected".to_string();
            return;
        }

        let tag_name = self.tag_name.clone();
        let value_str = self.tag_value.clone();
        let value_str_for_log = value_str.clone();

        let value = match self.tag_type {
            TagType::Dint => match value_str.parse::<i32>() {
                Ok(v) => PlcValue::Dint(v),
                Err(_) => {
                    self.tag_result = "Invalid DINT value".to_string();
                    return;
                }
            },
            TagType::Real => match value_str.parse::<f32>() {
                Ok(v) => PlcValue::Real(v),
                Err(_) => {
                    self.tag_result = "Invalid REAL value".to_string();
                    return;
                }
            },
            TagType::Bool => match value_str.to_lowercase().as_str() {
                "true" | "1" | "on" => PlcValue::Bool(true),
                "false" | "0" | "off" => PlcValue::Bool(false),
                _ => {
                    self.tag_result = "Invalid BOOL value (use true/false)".to_string();
                    return;
                }
            },
            TagType::Int => match value_str.parse::<i16>() {
                Ok(v) => PlcValue::Int(v),
                Err(_) => {
                    self.tag_result = "Invalid INT value".to_string();
                    return;
                }
            },
            TagType::String => PlcValue::String(value_str),
            TagType::Udt => {
                self.tag_result = "UDT writes require reading first to get symbol_id".to_string();
                return;
            }
        };

        self.add_log(format!(
            "Writing {:?} = {} to {}",
            value, value_str_for_log, tag_name
        ));

        let client = self.client.as_ref().unwrap().clone();
        let result = self.rt.block_on(async {
            let mut client_guard = client.lock().await;
            let write_result = client_guard.write_tag(&tag_name, value).await;
            drop(client_guard);
            write_result
        });

        match result {
            Ok(_) => {
                self.tag_result = "✅ Write successful".to_string();
                self.add_log(format!("✅ Wrote {} = {}", tag_name, value_str_for_log));
            }
            Err(e) => {
                self.tag_result = format!("❌ Error: {}", e);
                self.add_log(format!("❌ Failed to write {}: {}", tag_name, e));
            }
        }
    }

    fn read_array_element(&mut self) {
        if !self.connected {
            self.array_result = "Not connected".to_string();
            return;
        }

        if self.array_name.is_empty() {
            self.array_result = "Please enter an array name".to_string();
            return;
        }

        let client = self.client.as_ref().unwrap().clone();
        let array_name = self.array_name.clone();
        let index = self.array_index;

        let tag_name = format!("{}[{}]", array_name, index);
        self.add_log(format!("Reading array element: {}", tag_name));
        self.array_result = "Reading...".to_string();

        let result = self.rt.block_on(async {
            let mut client_guard = client.lock().await;
            let read_result = client_guard.read_tag(&tag_name).await;
            drop(client_guard);
            read_result
        });

        match result {
            Ok(value) => {
                let result_str = format!("✅ {:?}", value);
                self.array_result = result_str.clone();
                self.add_log(format!("✅ Read {}: {:?}", tag_name, value));
                println!("[DESKTOP_APP] Successfully read {}: {:?}", tag_name, value);
            }
            Err(e) => {
                let error_str = format!("❌ Error: {}", e);
                self.array_result = error_str.clone();
                self.add_log(format!("❌ Failed to read {}: {}", tag_name, e));
                println!("[DESKTOP_APP] Error reading {}: {}", tag_name, e);
            }
        }
    }

    fn write_array_element(&mut self) {
        if !self.connected {
            self.array_result = "Not connected".to_string();
            return;
        }

        let array_name = self.array_name.clone();
        let index = self.array_index;
        let value_str = self.array_value.clone();

        // Try to determine type from array name
        let value = if array_name.to_uppercase().contains("DINT") {
            match value_str.parse::<i32>() {
                Ok(v) => PlcValue::Dint(v),
                Err(_) => {
                    self.array_result = "Invalid DINT value".to_string();
                    return;
                }
            }
        } else if array_name.to_uppercase().contains("REAL") {
            match value_str.parse::<f32>() {
                Ok(v) => PlcValue::Real(v),
                Err(_) => {
                    self.array_result = "Invalid REAL value".to_string();
                    return;
                }
            }
        } else if array_name.to_uppercase().contains("BOOL") {
            match value_str.to_lowercase().as_str() {
                "true" | "1" | "on" => PlcValue::Bool(true),
                "false" | "0" | "off" => PlcValue::Bool(false),
                _ => {
                    self.array_result = "Invalid BOOL value".to_string();
                    return;
                }
            }
        } else if array_name.to_uppercase().contains("INT") {
            match value_str.parse::<i16>() {
                Ok(v) => PlcValue::Int(v),
                Err(_) => {
                    self.array_result = "Invalid INT value".to_string();
                    return;
                }
            }
        } else {
            // Default to DINT
            match value_str.parse::<i32>() {
                Ok(v) => PlcValue::Dint(v),
                Err(_) => {
                    self.array_result = "Invalid value".to_string();
                    return;
                }
            }
        };

        let tag_name = format!("{}[{}]", array_name, index);
        self.add_log(format!(
            "Writing {:?} = {} to {}",
            value, value_str, tag_name
        ));

        let client = self.client.as_ref().unwrap().clone();
        let result = self.rt.block_on(async {
            let mut client_guard = client.lock().await;
            let write_result = client_guard.write_tag(&tag_name, value).await;
            drop(client_guard);
            write_result
        });

        match result {
            Ok(_) => {
                self.array_result = "✅ Write successful".to_string();
                self.add_log(format!("✅ Wrote {} = {}", tag_name, value_str));
            }
            Err(e) => {
                self.array_result = format!("❌ Error: {}", e);
                self.add_log(format!("❌ Failed to write {}: {}", tag_name, e));
            }
        }
    }

    fn read_udt(&mut self) {
        if !self.connected {
            self.udt_result = "Not connected".to_string();
            return;
        }

        let client = self.client.as_ref().unwrap().clone();
        let udt_name = self.udt_name.clone();

        self.add_log(format!("Reading UDT: {}", udt_name));
        let result = self.rt.block_on(async {
            let mut client_guard = client.lock().await;
            let read_result = client_guard.read_tag(&udt_name).await;
            drop(client_guard);
            read_result
        });

        match result {
            Ok(PlcValue::Udt(udt_data)) => {
                self.udt_result = format!(
                    "✅ UDT read: symbol_id={}, data_size={} bytes",
                    udt_data.symbol_id,
                    udt_data.data.len()
                );
                self.add_log(format!(
                    "✅ Read UDT {}: symbol_id={}, {} bytes",
                    udt_name,
                    udt_data.symbol_id,
                    udt_data.data.len()
                ));
            }
            Ok(value) => {
                self.udt_result = format!("⚠️ Not a UDT: {:?}", value);
                self.add_log(format!("⚠️ {} is not a UDT: {:?}", udt_name, value));
            }
            Err(e) => {
                self.udt_result = format!("❌ Error: {}", e);
                self.add_log(format!("❌ Failed to read UDT {}: {}", udt_name, e));
            }
        }
    }

    fn read_udt_member(&mut self) {
        if !self.connected {
            self.udt_member_result = "Not connected".to_string();
            return;
        }

        let member_path = self.udt_member_path.clone();

        self.add_log(format!("Reading UDT member: {}", member_path));

        // Parse tag name and member path
        let parts: Vec<&str> = member_path.split('.').collect();
        if parts.len() < 2 {
            self.udt_member_result = "Invalid format: use 'TagName.MemberName'".to_string();
            return;
        }

        let client = self.client.as_ref().unwrap().clone();

        let result: Result<Option<PlcValue>, String> = self.rt.block_on(async {
            let mut client_guard = client.lock().await;

            // Try direct read first
            match client_guard.read_tag(&member_path).await {
                Ok(value) => {
                    return Ok(Some(value));
                }
                Err(_) => {
                    // Fall back to reading full UDT and extracting member
                }
            }

            // Read full UDT - for now, just return an error message
            // UDT member parsing requires UserDefinedType conversion which is complex
            // Direct tag access should work for most cases
            Err(format!(
                "UDT member parsing not fully implemented. Try direct tag access: {}",
                member_path
            ))
        });

        match result {
            Ok(Some(value)) => {
                self.udt_member_result = format!("✅ {:?}", value);
                self.add_log(format!("✅ Read {}: {:?}", member_path, value));
            }
            Ok(None) => {
                self.udt_member_result = "Unexpected result".to_string();
            }
            Err(e) => {
                self.udt_member_result = format!("❌ {}", e);
                self.add_log(format!("❌ Failed to read {}: {}", member_path, e));
            }
        }
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🦀 Rust EtherNet/IP Desktop Application");

            ui.separator();

            // Connection Section
            ui.group(|ui| {
                ui.heading("🔌 Connection");
                ui.horizontal(|ui| {
                    ui.label("PLC Address:");
                    ui.text_edit_singleline(&mut self.plc_address)
                        .on_hover_text("e.g., 192.168.1.100 or 192.168.1.100:44818");
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_route_path, "Use Route Path (ControlLogix)");
                    if self.use_route_path {
                        ui.label("CPU Slot:");
                        ui.add(egui::Slider::new(&mut self.cpu_slot, 0..=31));
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect();
                    }
                    if ui.button("Disconnect").clicked() {
                        self.disconnect();
                    }
                    ui.label(&self.connection_status);
                });
            });

            ui.separator();

            // Tabbed interface
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, 0, "Tag Operations");
                ui.selectable_value(&mut self.selected_tab, 1, "Array Operations");
                ui.selectable_value(&mut self.selected_tab, 2, "UDT Operations");
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Tag Operations
                if self.selected_tab == 0 {
                    ui.group(|ui| {
                        ui.heading("📖 Tag Operations");
                        ui.horizontal(|ui| {
                            ui.label("Tag Name:");
                            ui.text_edit_singleline(&mut self.tag_name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.selectable_value(&mut self.tag_type, TagType::Dint, "DINT");
                            ui.selectable_value(&mut self.tag_type, TagType::Real, "REAL");
                            ui.selectable_value(&mut self.tag_type, TagType::Bool, "BOOL");
                            ui.selectable_value(&mut self.tag_type, TagType::Int, "INT");
                            ui.selectable_value(&mut self.tag_type, TagType::String, "STRING");
                            ui.selectable_value(&mut self.tag_type, TagType::Udt, "UDT");
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Read").clicked() {
                                self.read_tag();
                            }
                            if ui.button("Write").clicked() {
                                self.write_tag();
                            }
                        });

                        if self.tag_type != TagType::Udt {
                            ui.horizontal(|ui| {
                                ui.label("Value:");
                                ui.text_edit_singleline(&mut self.tag_value);
                            });
                        }

                        ui.label(&self.tag_result);
                    });
                }

                ui.separator();

                // Array Operations
                if self.selected_tab == 1 {
                    ui.group(|ui| {
                        ui.heading("📊 Array Operations");
                        ui.horizontal(|ui| {
                            ui.label("Array Name:");
                            ui.text_edit_singleline(&mut self.array_name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Index:");
                            ui.add(egui::Slider::new(&mut self.array_index, 0..=100));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut self.array_value);
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Read Element").clicked() {
                                self.read_array_element();
                                ctx.request_repaint();
                            }
                            if ui.button("Write Element").clicked() {
                                self.write_array_element();
                                ctx.request_repaint();
                            }
                        });

                        ui.label(&self.array_result);
                    });
                }

                ui.separator();

                // UDT Operations
                if self.selected_tab == 2 {
                    ui.group(|ui| {
                        ui.heading("🏗️ UDT Operations");
                        ui.horizontal(|ui| {
                            ui.label("UDT Name:");
                            ui.text_edit_singleline(&mut self.udt_name);
                        });

                        if ui.button("Read UDT").clicked() {
                            self.read_udt();
                            ctx.request_repaint();
                        }

                        ui.label(&self.udt_result);

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Member Path:");
                            ui.text_edit_singleline(&mut self.udt_member_path)
                                .on_hover_text("e.g., gTestUDT.Member1_DINT");
                        });

                        if ui.button("Read UDT Member").clicked() {
                            self.read_udt_member();
                            ctx.request_repaint();
                        }

                        ui.label(&self.udt_member_result);

                        ui.separator();

                        ui.label("⚠️ Limitations:");
                        ui.label("• Cannot write directly to UDT array element members");
                        ui.label("  (e.g., gTestUDT_Array[0].Member1_DINT)");
                        ui.label(
                            "• Workaround: Read entire UDT array element, modify, then write back",
                        );
                    });
                }
            });

            ui.separator();

            // Log Section
            ui.group(|ui| {
                ui.heading("📋 Activity Log");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for msg in &self.log_messages {
                            ui.label(msg);
                        }
                    });
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 1000.0])
            .with_title("Rust EtherNet/IP Desktop App"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust EtherNet/IP Desktop App",
        options,
        Box::new(|_cc| Ok(Box::new(DesktopApp::default()))),
    )
}
