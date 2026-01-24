using System;
using System.Collections.Generic;
using RustEtherNetIp;

namespace TestCellNestDataUdt
{
    /// <summary>
    /// C# Test for reading Cell_NestData[90] nested UDT array structure
    /// 
    /// This test verifies that the C# wrapper can correctly read:
    /// - Array of UDT elements: Cell_NestData[90]
    /// - Nested UDT members: Cell_NestData[90].PartData
    /// - Nested array members: Cell_NestData[90].PartData.PlungerInsertion[0]
    /// 
    /// Run with: dotnet run
    /// 
    /// Prerequisites:
    /// - Cell_NestData tag must exist in the PLC (array of 100 UDT elements)
    /// - PLC must be accessible at 192.168.1.101:44818
    /// - CPU slot 0 (CompactLogix) or adjust CPU_SLOT constant
    /// </summary>
    class Program
    {
        private const string PLC_ADDRESS = "192.168.1.101:44818";
        private const byte CPU_SLOT = 0; // CompactLogix CPU in Slot 0

        static void Main(string[] args)
        {
            Console.WriteLine("╔══════════════════════════════════════════════════════════════════════════════╗");
            Console.WriteLine("║  Cell_NestData UDT Array Reading Test (C# Wrapper)                          ║");
            Console.WriteLine("║  Tests reading: Cell_NestData[90] with nested PartData UDT                 ║");
            Console.WriteLine("╚══════════════════════════════════════════════════════════════════════════════╝");
            Console.WriteLine();

            Console.WriteLine($"🔌 Connecting to PLC at {PLC_ADDRESS} (slot {CPU_SLOT})...");

            using var client = new EtherNetIpClient();
            
            // Connect with route path
            var routePath = new RoutePath().AddSlot(CPU_SLOT);
            if (!client.ConnectWithRoute(PLC_ADDRESS, routePath))
            {
                Console.WriteLine("❌ Failed to connect to PLC");
                return;
            }

            Console.WriteLine("✅ Connected successfully!");
            Console.WriteLine();

            // TEST 1: Read entire Cell_NestData[90] UDT
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 1: Read entire Cell_NestData[90] UDT");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            string tagPath = "Cell_NestData[90]";
            Console.WriteLine($"\n📖 Reading: {tagPath}");
            
            try
            {
                var udtValue = client.ReadUdt(tagPath);
                Console.WriteLine("✅ Successfully read UDT!");
                Console.WriteLine($"   Type: {udtValue.Type}");
                Console.WriteLine($"   IsUdt: {udtValue.IsUdt}");
                
                if (udtValue.IsUdt)
                {
                    var udtData = udtValue.UdtData;
                    Console.WriteLine($"   Symbol ID: {udtData.SymbolId}");
                    Console.WriteLine($"   Data size: {udtData.Data.Length} bytes");
                    Console.WriteLine($"   Raw data (first 64 bytes): {BitConverter.ToString(udtData.Data, 0, Math.Min(64, udtData.Data.Length))}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"❌ Failed to read {tagPath}: {ex.Message}");
            }

            // TEST 2: Read nested UDT member - Cell_NestData[90].PartData
            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 2: Read nested UDT member - Cell_NestData[90].PartData");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            tagPath = "Cell_NestData[90].PartData";
            Console.WriteLine($"\n📖 Reading: {tagPath}");
            
            try
            {
                var udtValue = client.ReadUdt(tagPath);
                Console.WriteLine("✅ Successfully read nested UDT PartData!");
                Console.WriteLine($"   Type: {udtValue.Type}");
                
                if (udtValue.IsUdt)
                {
                    var udtData = udtValue.UdtData;
                    Console.WriteLine($"   Symbol ID: {udtData.SymbolId}");
                    Console.WriteLine($"   Data size: {udtData.Data.Length} bytes");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"❌ Failed to read {tagPath}: {ex.Message}");
            }

            // TEST 3: Read individual PartData members
            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 3: Read individual PartData members");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            var members = new List<string>
            {
                "Cell_NestData[90].PartData.Temp_PreHeatZone1",
                "Cell_NestData[90].PartData.Temp_PreHeatZone2",
                "Cell_NestData[90].PartData.Time_PreHeat",
                "Cell_NestData[90].PartData.Temp_HeatZone1",
                "Cell_NestData[90].PartData.Temp_HeatZone2",
                "Cell_NestData[90].PartData.Time_Heat",
                "Cell_NestData[90].PartData.Time_Cooling",
            };

            foreach (var member in members)
            {
                Console.WriteLine($"\n📖 Reading: {member}");
                try
                {
                    // Use ReadTagWithDetails for complex nested paths - it uses the generic eip_read_tag
                    var result = client.ReadTagWithDetails(member);
                    if (result.Success && result.Value != null)
                    {
                        if (result.Value.Type == PlcValueType.Real)
                        {
                            var value = result.Value.As<float>();
                            Console.WriteLine($"   ✅ Value: {value}");
                        }
                        else
                        {
                            Console.WriteLine($"   ❌ Error: Expected REAL but got {result.Value.Type}. Value: {result.Value}");
                        }
                    }
                    else
                    {
                        var errorMsg = !string.IsNullOrEmpty(result.ErrorMessage) ? result.ErrorMessage : "Unknown error";
                        Console.WriteLine($"   ❌ Error: Read failed. Success={result.Success}, Error: {errorMsg}");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"   ❌ Error: {ex.Message}");
                }
            }

            // TEST 4: Read nested array member - PlungerInsertion[0-3]
            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 4: Read nested array member - PlungerInsertion[0-3]");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            for (int i = 0; i < 4; i++)
            {
                tagPath = $"Cell_NestData[90].PartData.PlungerInsertion[{i}]";
                Console.WriteLine($"\n📖 Reading: {tagPath}");
                try
                {
                    // Use ReadTagWithDetails for complex nested paths - it uses the generic eip_read_tag
                    var result = client.ReadTagWithDetails(tagPath);
                    if (result.Success && result.Value != null)
                    {
                        if (result.Value.Type == PlcValueType.Real)
                        {
                            var value = result.Value.As<float>();
                            Console.WriteLine($"   ✅ Value: {value}");
                        }
                        else
                        {
                            Console.WriteLine($"   ❌ Error: Expected REAL but got {result.Value.Type}. Value: {result.Value}");
                        }
                    }
                    else
                    {
                        var errorMsg = !string.IsNullOrEmpty(result.ErrorMessage) ? result.ErrorMessage : "Unknown error";
                        Console.WriteLine($"   ❌ Error: Read failed. Success={result.Success}, Error: {errorMsg}");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"   ❌ Error: {ex.Message}");
                }
            }

            // TEST 5: Read other PartData members
            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 5: Read other PartData members");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            var otherMembers = new List<string>
            {
                "Cell_NestData[90].PartData.Vision_AngleBody_1",
                "Cell_NestData[90].PartData.Vision_PlungerDist",
                "Cell_NestData[90].PartData.Vision_CapPres",
                "Cell_NestData[90].PartData.Vision_AngleBody_2",
                "Cell_NestData[90].PartData.Vision_PlungerDist_2",
                "Cell_NestData[90].PartData.Time_FillDecTime",
                "Cell_NestData[90].PartData.Weigh_BodyPlunger",
                "Cell_NestData[90].PartData.Weigh_Cap",
                "Cell_NestData[90].PartData.Weigh_Final",
            };

            foreach (var member in otherMembers)
            {
                Console.WriteLine($"\n📖 Reading: {member}");
                try
                {
                    // Use ReadTagWithDetails for complex nested paths - it uses the generic eip_read_tag
                    var result = client.ReadTagWithDetails(member);
                    if (result.Success && result.Value != null)
                    {
                        if (result.Value.Type == PlcValueType.Real)
                        {
                            var value = result.Value.As<float>();
                            Console.WriteLine($"   ✅ Value: {value}");
                        }
                        else
                        {
                            Console.WriteLine($"   ❌ Error: Expected REAL but got {result.Value.Type}. Value: {result.Value}");
                        }
                    }
                    else
                    {
                        var errorMsg = !string.IsNullOrEmpty(result.ErrorMessage) ? result.ErrorMessage : "Unknown error";
                        Console.WriteLine($"   ❌ Error: Read failed. Success={result.Success}, Error: {errorMsg}");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"   ❌ Error: {ex.Message}");
                }
            }

            // TEST 6: Read top-level Cell_NestData[90] members
            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("TEST 6: Read top-level Cell_NestData[90] members");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
            
            var topLevelMembers = new List<(string Name, string Type)>
            {
                ("Cell_NestData[90].ModelNumber", "String"),
                ("Cell_NestData[90].SerialNumber", "String"),
                ("Cell_NestData[90].LotNo", "String"),
                ("Cell_NestData[90].LastStationWorked", "Dint"),
                ("Cell_NestData[90].PartStatus", "Dint"),
                ("Cell_NestData[90].StationPartStatus", "Dint"),
                ("Cell_NestData[90].IndexPositionNestNumber", "Dint"),
                ("Cell_NestData[90].ReworkActive", "Bool"),
                ("Cell_NestData[90].MasterPart", "Bool"),
            };

            foreach (var (memberName, memberType) in topLevelMembers)
            {
                Console.WriteLine($"\n📖 Reading: {memberName}");
                try
                {
                    // Use ReadTagWithDetails for complex nested paths - it uses the generic eip_read_tag
                    var result = client.ReadTagWithDetails(memberName);
                    if (!result.Success)
                    {
                        var errorMsg = !string.IsNullOrEmpty(result.ErrorMessage) ? result.ErrorMessage : "Unknown error";
                        Console.WriteLine($"   ❌ Error: Read failed. Error: {errorMsg}");
                        continue;
                    }

                    switch (memberType)
                    {
                        case "String":
                            if (result.Value.Type == PlcValueType.String)
                            {
                                var stringValue = result.Value.As<string>();
                                Console.WriteLine($"   ✅ Value: {stringValue}");
                            }
                            else
                            {
                                Console.WriteLine($"   ❌ Error: Expected String but got {result.Value.Type}");
                            }
                            break;
                        case "Dint":
                            if (result.Value.Type == PlcValueType.Dint)
                            {
                                var dintValue = result.Value.As<int>();
                                Console.WriteLine($"   ✅ Value: {dintValue}");
                            }
                            else
                            {
                                Console.WriteLine($"   ❌ Error: Expected Dint but got {result.Value.Type}");
                            }
                            break;
                        case "Bool":
                            if (result.Value.Type == PlcValueType.Bool)
                            {
                                var boolValue = result.Value.As<bool>();
                                Console.WriteLine($"   ✅ Value: {boolValue}");
                            }
                            else
                            {
                                Console.WriteLine($"   ❌ Error: Expected Bool but got {result.Value.Type}");
                            }
                            break;
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"   ❌ Error: {ex.Message}");
                }
            }

            Console.WriteLine("\n═══════════════════════════════════════════════════════════════════════════════");
            Console.WriteLine("Test completed!");
            Console.WriteLine("═══════════════════════════════════════════════════════════════════════════════");
        }
    }
}
