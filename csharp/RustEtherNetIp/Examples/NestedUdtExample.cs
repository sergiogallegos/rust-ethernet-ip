using System;
using System.Collections.Generic;

namespace RustEtherNetIp.Examples
{
    /// <summary>
    /// Comprehensive example demonstrating nested UDT support in the C# wrapper.
    /// This example shows how to work with complex nested UDT structures.
    /// </summary>
    public class NestedUdtExample
    {
        public static void RunExample()
        {
            Console.WriteLine("🏗️ Nested UDT Example - Rust EtherNet/IP C# Wrapper");
            Console.WriteLine("====================================================\n");

            using var client = new EtherNetIpClient();
            
            if (!client.Connect("192.168.0.1:44818"))
            {
                Console.WriteLine("❌ Failed to connect to PLC");
                return;
            }

            Console.WriteLine("✅ Connected to PLC\n");

            try
            {
                // Example 1: Simple Nested UDT
                Console.WriteLine("📝 Example 1: Simple Nested UDT");
                CreateSimpleNestedUdt(client);

                // Example 2: Complex Multi-Level Nested UDT
                Console.WriteLine("\n📝 Example 2: Complex Multi-Level Nested UDT");
                CreateComplexNestedUdt(client);

                // Example 3: Working with Nested Members
                Console.WriteLine("\n📝 Example 3: Working with Nested Members");
                WorkWithNestedMembers(client);

                // Example 4: Batch Operations with UDTs
                Console.WriteLine("\n📝 Example 4: Batch Operations with UDTs");
                BatchOperationsWithUdt(client);

                // Example 5: Real-World Motor Control System
                Console.WriteLine("\n📝 Example 5: Real-World Motor Control System");
                MotorControlSystemExample(client);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"❌ Error: {ex.Message}");
            }
        }

        /// <summary>
        /// Example 1: Simple nested UDT with two levels
        /// </summary>
        private static void CreateSimpleNestedUdt(EtherNetIpClient client)
        {
            // Create a motor UDT with nested status and config
            var motorData = new Dictionary<string, PlcValue>
            {
                ["MotorID"] = PlcValue.Dint(1),
                ["Name"] = PlcValue.String("MainMotor"),
                
                // Nested Status UDT
                ["Status"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Running"] = PlcValue.Bool(true),
                    ["Fault"] = PlcValue.Bool(false),
                    ["ErrorCode"] = PlcValue.Dint(0)
                }),
                
                // Nested Config UDT
                ["Config"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Speed"] = PlcValue.Real(1750.0f),
                    ["Acceleration"] = PlcValue.Real(100.0f),
                    ["MaxCurrent"] = PlcValue.Real(15.5f)
                })
            };

            // Write the nested UDT
            client.WriteUdt("MotorData", PlcValue.Udt(motorData));
            Console.WriteLine("✅ Written simple nested UDT");

            // Read it back
            var readData = client.ReadUdt("MotorData");
            var members = readData.UdtMembers;
            if (members != null)
            {
                Console.WriteLine($"📖 Read UDT with {members.Count} top-level members");
            }
            else
            {
                Console.WriteLine("📖 Read UDT in raw data format");
            }

            // Access nested values
            var status = readData.GetNestedValue("Status");
            if (status?.IsUdt == true)
            {
                var isRunning = status.GetNestedValue("Running")?.As<bool>() ?? false;
                Console.WriteLine($"   Motor Running: {isRunning}");
            }
        }

        /// <summary>
        /// Example 2: Complex multi-level nested UDT
        /// </summary>
        private static void CreateComplexNestedUdt(EtherNetIpClient client)
        {
            // Create a production line UDT with multiple levels of nesting
            var productionLine = new Dictionary<string, PlcValue>
            {
                ["LineID"] = PlcValue.Dint(1),
                ["LineName"] = PlcValue.String("Assembly Line 1"),
                
                // Nested Station UDT
                ["Station1"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["StationID"] = PlcValue.Dint(1),
                    ["Status"] = PlcValue.String("Active"),
                    
                    // Nested Motor UDT within Station
                    ["Motor"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                    {
                        ["Running"] = PlcValue.Bool(true),
                        ["Speed"] = PlcValue.Real(1500.0f),
                        ["Temperature"] = PlcValue.Real(45.2f),
                        
                        // Nested Diagnostics UDT within Motor
                        ["Diagnostics"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                        {
                            ["Vibration"] = PlcValue.Real(0.5f),
                            ["OperatingHours"] = PlcValue.Udint(1250),
                            ["LastMaintenance"] = PlcValue.Dint(20240115)
                        })
                    }),
                    
                    // Nested Sensor UDT within Station
                    ["Sensor"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                    {
                        ["Type"] = PlcValue.String("Proximity"),
                        ["Value"] = PlcValue.Real(12.5f),
                        ["Calibrated"] = PlcValue.Bool(true)
                    })
                })
            };

            // Write the complex nested UDT
            client.WriteUdt("ProductionLine", PlcValue.Udt(productionLine));
            Console.WriteLine("✅ Written complex multi-level nested UDT");

            // Read and display structure
            var lineData = client.ReadUdt("ProductionLine");
            Console.WriteLine($"📖 Production Line: {lineData.GetNestedValue("LineName")?.As<string>()}");
            
            var station = lineData.GetNestedValue("Station1");
            if (station?.IsUdt == true)
            {
                Console.WriteLine($"   Station Status: {station.GetNestedValue("Status")?.As<string>()}");
                
                var motor = station.GetNestedValue("Motor");
                if (motor?.IsUdt == true)
                {
                    Console.WriteLine($"   Motor Running: {motor.GetNestedValue("Running")?.As<bool>()}");
                    Console.WriteLine($"   Motor Speed: {motor.GetNestedValue("Speed")?.As<float>()} RPM");
                }
            }
        }

        /// <summary>
        /// Example 3: Working with nested members using dot notation
        /// </summary>
        private static void WorkWithNestedMembers(EtherNetIpClient client)
        {
            // Read individual nested members using dot notation
            var motorRunning = client.GetUdtMember("MotorData", "Status.Running");
            Console.WriteLine($"Motor Running (via dot notation): {motorRunning?.As<bool>()}");

            var motorSpeed = client.GetUdtMember("MotorData", "Config.Speed");
            Console.WriteLine($"Motor Speed (via dot notation): {motorSpeed?.As<float>()} RPM");

            // Update individual nested members
            client.SetUdtMember("MotorData", "Status.Running", PlcValue.Bool(false));
            client.SetUdtMember("MotorData", "Config.Speed", PlcValue.Real(2000.0f));

            Console.WriteLine("✅ Updated nested members using dot notation");

            // Verify the changes
            var updatedRunning = client.GetUdtMember("MotorData", "Status.Running");
            var updatedSpeed = client.GetUdtMember("MotorData", "Config.Speed");
            
            Console.WriteLine($"Updated Motor Running: {updatedRunning?.As<bool>()}");
            Console.WriteLine($"Updated Motor Speed: {updatedSpeed?.As<float>()} RPM");
        }

        /// <summary>
        /// Example 4: Batch operations with UDTs
        /// </summary>
        private static void BatchOperationsWithUdt(EtherNetIpClient client)
        {
            // Create multiple UDTs for batch operations
            var motor1 = CreateMotorUdt(1, "Motor1", 1500.0f, true);
            var motor2 = CreateMotorUdt(2, "Motor2", 1800.0f, false);
            var motor3 = CreateMotorUdt(3, "Motor3", 1200.0f, true);

            // Write multiple UDTs
            client.WriteUdt("BatchMotor1", motor1);
            client.WriteUdt("BatchMotor2", motor2);
            client.WriteUdt("BatchMotor3", motor3);

            Console.WriteLine("✅ Written 3 UDTs for batch operations");

            // Read multiple UDTs
            var tags = new[] { "BatchMotor1", "BatchMotor2", "BatchMotor3" };
            var results = client.ReadTagsBatch(tags);

            Console.WriteLine("📖 Batch read results:");
            foreach (var (tagName, result) in results)
            {
                if (result.Success && result.Value is PlcValue udtValue && udtValue.IsUdt)
                {
                    var motorName = udtValue.GetNestedValue("Name")?.As<string>() ?? "Unknown";
                    var speed = udtValue.GetNestedValue("Config.Speed")?.As<float>() ?? 0.0f;
                    var running = udtValue.GetNestedValue("Status.Running")?.As<bool>() ?? false;
                    
                    Console.WriteLine($"   {tagName}: {motorName} - Speed: {speed} RPM, Running: {running}");
                }
            }
        }

        /// <summary>
        /// Example 5: Real-world motor control system
        /// </summary>
        private static void MotorControlSystemExample(EtherNetIpClient client)
        {
            // Create a complete motor control system with multiple nested UDTs
            var controlSystem = new Dictionary<string, PlcValue>
            {
                ["SystemID"] = PlcValue.Dint(100),
                ["SystemName"] = PlcValue.String("Main Control System"),
                ["Version"] = PlcValue.String("1.2.3"),
                
                // System Status UDT
                ["SystemStatus"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Online"] = PlcValue.Bool(true),
                    ["Mode"] = PlcValue.String("Auto"),
                    ["AlarmCount"] = PlcValue.Dint(0),
                    ["LastUpdate"] = PlcValue.Dint(20240115)
                }),
                
                // Motors Array (simulated as individual UDTs)
                ["Motor1"] = CreateMotorUdt(1, "Conveyor Motor", 1000.0f, true),
                ["Motor2"] = CreateMotorUdt(2, "Lift Motor", 800.0f, false),
                ["Motor3"] = CreateMotorUdt(3, "Gripper Motor", 500.0f, true),
                
                // Safety System UDT
                ["Safety"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["EmergencyStop"] = PlcValue.Bool(false),
                    ["LightCurtain"] = PlcValue.Bool(true),
                    ["DoorOpen"] = PlcValue.Bool(false),
                    
                    // Nested Safety Diagnostics
                    ["Diagnostics"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                    {
                        ["LastTest"] = PlcValue.Dint(20240101),
                        ["TestResult"] = PlcValue.String("PASS"),
                        ["NextTest"] = PlcValue.Dint(20240201)
                    })
                })
            };

            // Write the complete system
            client.WriteUdt("ControlSystem", PlcValue.Udt(controlSystem));
            Console.WriteLine("✅ Written complete motor control system");

            // Monitor the system
            var system = client.ReadUdt("ControlSystem");
            Console.WriteLine($"🏭 Control System: {system.GetNestedValue("SystemName")?.As<string>()}");
            
            var systemStatus = system.GetNestedValue("SystemStatus");
            if (systemStatus?.IsUdt == true)
            {
                var online = systemStatus.GetNestedValue("Online")?.As<bool>() ?? false;
                var mode = systemStatus.GetNestedValue("Mode")?.As<string>() ?? "Unknown";
                Console.WriteLine($"   Status: Online={online}, Mode={mode}");
            }

            // Check individual motors
            for (int i = 1; i <= 3; i++)
            {
                var motor = system.GetNestedValue($"Motor{i}");
                if (motor?.IsUdt == true)
                {
                    var name = motor.GetNestedValue("Name")?.As<string>() ?? "Unknown";
                    var running = motor.GetNestedValue("Status.Running")?.As<bool>() ?? false;
                    var speed = motor.GetNestedValue("Config.Speed")?.As<float>() ?? 0.0f;
                    Console.WriteLine($"   Motor {i}: {name} - Running: {running}, Speed: {speed} RPM");
                }
            }

            // Check safety system
            var safety = system.GetNestedValue("Safety");
            if (safety?.IsUdt == true)
            {
                var emergencyStop = safety.GetNestedValue("EmergencyStop")?.As<bool>() ?? true;
                var lightCurtain = safety.GetNestedValue("LightCurtain")?.As<bool>() ?? false;
                Console.WriteLine($"   Safety: Emergency Stop={emergencyStop}, Light Curtain={lightCurtain}");
            }
        }

        /// <summary>
        /// Helper method to create a motor UDT
        /// </summary>
        private static PlcValue CreateMotorUdt(int id, string name, float speed, bool running)
        {
            return PlcValue.Udt(new Dictionary<string, PlcValue>
            {
                ["MotorID"] = PlcValue.Dint(id),
                ["Name"] = PlcValue.String(name),
                
                ["Status"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Running"] = PlcValue.Bool(running),
                    ["Fault"] = PlcValue.Bool(false),
                    ["ErrorCode"] = PlcValue.Dint(0)
                }),
                
                ["Config"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Speed"] = PlcValue.Real(speed),
                    ["Acceleration"] = PlcValue.Real(100.0f),
                    ["MaxCurrent"] = PlcValue.Real(15.5f)
                }),
                
                ["Diagnostics"] = PlcValue.Udt(new Dictionary<string, PlcValue>
                {
                    ["Temperature"] = PlcValue.Real(45.0f + (id * 2.5f)),
                    ["Vibration"] = PlcValue.Real(0.3f + (id * 0.1f)),
                    ["OperatingHours"] = PlcValue.Udint((uint)(1000 + id * 100))
                })
            });
        }
    }
}
