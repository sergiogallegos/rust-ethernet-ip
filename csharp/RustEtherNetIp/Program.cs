// Program.cs - Demo application only
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading.Tasks;
using RustEtherNetIp;

namespace RustEtherNetIp
{
    /// <summary>
    /// Example program demonstrating EtherNet/IP client usage including STRING operations and batch operations.
    /// </summary>
    internal class Program
    {
        private static async Task Main(string[] args)
        {
            Console.WriteLine("EtherNet/IP Client Test Program");
            Console.WriteLine("==============================");

            // Get PLC address from command line or use default
            string plcAddress = args.Length > 0 ? args[0] : "192.168.0.1:44818";
            Console.WriteLine($"Connecting to PLC at {plcAddress}...");

            try
            {
                using var client = new EtherNetIpClient();
                if (!client.Connect(plcAddress))
                {
                    Console.WriteLine("Failed to connect to PLC");
                    return;
                }

                Console.WriteLine("Connected successfully!");
                Console.WriteLine("\nTesting basic read/write operations...");

                // Test basic read/write operations
                try
                {
                    // Write some test values
                    client.WriteBool("_IO_EM_DI00", true);
                    client.WriteDint("_IO_EM_DI01", 42);
                    client.WriteReal("_IO_EM_DI02", 3.14159f);
                    client.WriteString("_IO_EM_DI03", "Hello from C#!");

                    // Read them back
                    bool boolValue = client.ReadBool("_IO_EM_DI00");
                    int dintValue = client.ReadDint("_IO_EM_DI01");
                    float realValue = client.ReadReal("_IO_EM_DI02");
                    string stringValue = client.ReadString("_IO_EM_DI03");

                    Console.WriteLine($"Read values:");
                    Console.WriteLine($"  _IO_EM_DI00: {boolValue}");
                    Console.WriteLine($"  _IO_EM_DI01: {dintValue}");
                    Console.WriteLine($"  _IO_EM_DI02: {realValue}");
                    Console.WriteLine($"  _IO_EM_DI03: {stringValue}");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error during basic read/write test: {ex.Message}");
                }

                Console.WriteLine("\nTesting batch operations...");

                // Test batch operations
                try
                {
                    var operations = new[]
                    {
                        BatchOperation.Read("_IO_EM_DI00"),
                        BatchOperation.Read("_IO_EM_DI01"),
                        BatchOperation.Read("_IO_EM_DI02"),
                        BatchOperation.Read("_IO_EM_DI03")
                    };

                    var results = client.ExecuteBatch(operations);
                    foreach (var result in results)
                    {
                        Console.WriteLine($"Batch read {result.TagName}: {(result.Success ? result.Value : "Failed")}");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error during batch operations test: {ex.Message}");
                }

                Console.WriteLine("\nTesting tag subscriptions...");

                // Test tag subscriptions
                try
                {
                    var subscriptionOptions = new SubscriptionOptions
                    {
                        PollIntervalMs = 100,
                        AutoReconnect = true,
                        MaxReconnectAttempts = 3
                    };

                    var boolSubscription = client.SubscribeToTag("_IO_EM_DI00", subscriptionOptions);
                    var dintSubscription = client.SubscribeToTag("_IO_EM_DI01", subscriptionOptions);

                    boolSubscription.ValueChanged += (sender, args) =>
                    {
                        Console.WriteLine($"_IO_EM_DI00 changed: {args.OldValue} -> {args.NewValue}");
                    };

                    dintSubscription.ValueChanged += (sender, args) =>
                    {
                        Console.WriteLine($"_IO_EM_DI01 changed: {args.OldValue} -> {args.NewValue}");
                    };

                    Console.WriteLine("Subscriptions active. Press any key to stop...");
                    Console.ReadKey();

                    // Clean up subscriptions
                    client.UnsubscribeFromAllTags();
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error during subscription test: {ex.Message}");
                }

                Console.WriteLine("\nTesting connection health...");

                // Test connection health
                try
                {
                    bool isHealthy = client.CheckHealth();
                    Console.WriteLine($"Connection health: {(isHealthy ? "Good" : "Poor")}");

                    if (client.CheckHealthDetailed())
                    {
                        Console.WriteLine("Detailed health check passed");
                    }
                    else
                    {
                        Console.WriteLine("Detailed health check failed");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error during health check: {ex.Message}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
            }

            Console.WriteLine("\nTest completed. Press any key to exit...");
            Console.ReadKey();
        }
    }
}
