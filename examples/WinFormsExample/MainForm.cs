using System;
using System.Collections.Generic;
using System.Drawing;
using System.Windows.Forms;
using RustEtherNetIp;
using System.Linq;
using System.Diagnostics;

namespace WinFormsExample
{
    public partial class MainForm : Form
    {
        private EtherNetIpClient? _plcClient;
        private bool _isConnected;
        private string _currentAddress = string.Empty;
        private System.Windows.Forms.Timer? _connectionMonitorTimer;
        private Dictionary<string, TagInfo> _tags = new();
        private const int MAX_RETRIES = 3;
        private const int RETRY_DELAY = 5000; // 5 seconds
        private int _retryCount = 0;
        private bool _isReconnecting = false;

        public MainForm()
        {
            InitializeComponent();
            InitializeCustomComponents();
            SetupTimers();
            UpdateConnectionStatus();
        }

        private void InitializeCustomComponents()
        {
            // Set form properties
            this.Text = "🦀 Rust EtherNet/IP - WinForms Demo with Batch Operations";
            this.Size = new Size(1400, 1000);
            this.StartPosition = FormStartPosition.CenterScreen;

            // Create main layout (vertical stack)
            var mainLayout = new TableLayoutPanel
            {
                Dock = DockStyle.Fill,
                ColumnCount = 1,
                RowCount = 3,
                Padding = new Padding(10)
            };
            mainLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 100));   // Header
            mainLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 100));     // Tab Control
            mainLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 200));    // Log panel

            // Header Panel
            var headerPanel = CreateHeaderPanel();
            mainLayout.Controls.Add(headerPanel, 0, 0);

            // Tab Control for different operation modes
            var tabControl = CreateTabControl();
            mainLayout.Controls.Add(tabControl, 0, 1);

            // Log Panel (bottom, full width)
            var logPanel = CreateLogPanel();
            mainLayout.Controls.Add(logPanel, 0, 2);

            // Add the main layout to the form
            this.Controls.Add(mainLayout);
        }

        private Panel CreateHeaderPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill };

            // PLC Address input
            var plcAddressLabel = new Label
            {
                Text = "PLC Address:",
                Location = new Point(10, 15),
                AutoSize = true
            };
            panel.Controls.Add(plcAddressLabel);

            var plcAddressTextBox = new TextBox
            {
                Name = "plcAddressTextBox",
                Location = new Point(100, 12),
                Size = new Size(200, 23),
                Text = "192.168.0.1:44818"
            };
            panel.Controls.Add(plcAddressTextBox);

            // Connect/Disconnect buttons
            var connectButton = new Button
            {
                Name = "connectButton",
                Text = "Connect",
                Location = new Point(320, 11),
                Size = new Size(100, 25),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White
            };
            connectButton.Click += ConnectButton_Click;
            panel.Controls.Add(connectButton);

            var disconnectButton = new Button
            {
                Name = "disconnectButton",
                Text = "Disconnect",
                Location = new Point(430, 11),
                Size = new Size(100, 25),
                BackColor = Color.FromArgb(239, 68, 68),
                ForeColor = Color.White,
                Enabled = false
            };
            disconnectButton.Click += DisconnectButton_Click;
            panel.Controls.Add(disconnectButton);

            // Connection status
            var statusLabel = new Label
            {
                Name = "statusLabel",
                Text = "Disconnected",
                Location = new Point(10, 50),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(239, 68, 68)
            };
            panel.Controls.Add(statusLabel);

            var sessionLabel = new Label
            {
                Name = "sessionLabel",
                Text = "Session: None",
                Location = new Point(10, 70),
                AutoSize = true
            };
            panel.Controls.Add(sessionLabel);

            return panel;
        }

        private TabControl CreateTabControl()
        {
            var tabControl = new TabControl
            {
                Dock = DockStyle.Fill,
                Name = "mainTabControl"
            };

            // Individual Operations Tab
            var individualTab = new TabPage("Individual Operations");
            individualTab.Controls.Add(CreateIndividualOperationsPanel());
            tabControl.TabPages.Add(individualTab);

            // Batch Operations Tab
            var batchTab = new TabPage("🚀 Batch Operations");
            batchTab.Controls.Add(CreateBatchOperationsPanel());
            tabControl.TabPages.Add(batchTab);

            // Performance Comparison Tab
            var performanceTab = new TabPage("📊 Performance Comparison");
            performanceTab.Controls.Add(CreatePerformancePanel());
            tabControl.TabPages.Add(performanceTab);

            // Batch Configuration Tab
            var configTab = new TabPage("⚙️ Batch Configuration");
            configTab.Controls.Add(CreateBatchConfigPanel());
            tabControl.TabPages.Add(configTab);

            return tabControl;
        }

        private Panel CreateIndividualOperationsPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            var layout = new TableLayoutPanel
            {
                Dock = DockStyle.Top,
                ColumnCount = 4,
                RowCount = 2,
                AutoSize = true,
                AutoSizeMode = AutoSizeMode.GrowAndShrink
            };
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 200));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 130));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 180));
            layout.ColumnStyles.Add(new ColumnStyle(SizeType.Absolute, 180));

            // Row 0: Tag discovery
            var discoverTextBox = new TextBox
            {
                Name = "discoverTextBox",
                PlaceholderText = "Enter tag name to discover"
            };
            layout.Controls.Add(discoverTextBox, 0, 0);

            var discoverButton = new Button
            {
                Name = "discoverButton",
                Text = "Discover",
                BackColor = Color.FromArgb(59, 130, 246),
                ForeColor = Color.White,
                Enabled = false
            };
            discoverButton.Click += DiscoverButton_Click;
            layout.Controls.Add(discoverButton, 1, 0);

            // Row 1: Tag operations
            var tagNameTextBox = new TextBox
            {
                Name = "tagNameTextBox",
                PlaceholderText = "Tag name"
            };
            layout.Controls.Add(tagNameTextBox, 0, 1);

            var dataTypeComboBox = new ComboBox
            {
                Name = "dataTypeComboBox",
                DropDownStyle = ComboBoxStyle.DropDownList
            };
            dataTypeComboBox.Items.AddRange(new[] { "BOOL", "SINT", "INT", "DINT", "LINT", "USINT", "UINT", "UDINT", "ULINT", "REAL", "LREAL", "STRING", "UDT" });
            dataTypeComboBox.SelectedIndex = 0;
            layout.Controls.Add(dataTypeComboBox, 1, 1);

            var tagValueTextBox = new TextBox
            {
                Name = "tagValueTextBox",
                PlaceholderText = "Value"
            };
            layout.Controls.Add(tagValueTextBox, 2, 1);

            var operationPanel = new Panel { Dock = DockStyle.Fill };
            var readButton = new Button
            {
                Name = "readButton",
                Text = "Read",
                Location = new Point(0, 0),
                Size = new Size(80, 25),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            readButton.Click += ReadButton_Click;
            operationPanel.Controls.Add(readButton);

            var writeButton = new Button
            {
                Name = "writeButton",
                Text = "Write",
                Location = new Point(90, 0),
                Size = new Size(80, 25),
                BackColor = Color.FromArgb(249, 115, 22),
                ForeColor = Color.White,
                Enabled = false
            };
            writeButton.Click += WriteButton_Click;
            operationPanel.Controls.Add(writeButton);

            layout.Controls.Add(operationPanel, 3, 1);

            panel.Controls.Add(layout);
            return panel;
        }

        private Panel CreateBatchOperationsPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            var batchTabControl = new TabControl
            {
                Dock = DockStyle.Fill,
                Name = "batchTabControl"
            };

            // Batch Read Tab
            var batchReadTab = new TabPage("Batch Read");
            batchReadTab.Controls.Add(CreateBatchReadPanel());
            batchTabControl.TabPages.Add(batchReadTab);

            // Batch Write Tab
            var batchWriteTab = new TabPage("Batch Write");
            batchWriteTab.Controls.Add(CreateBatchWritePanel());
            batchTabControl.TabPages.Add(batchWriteTab);

            // Mixed Batch Tab
            var mixedBatchTab = new TabPage("Mixed Operations");
            mixedBatchTab.Controls.Add(CreateMixedBatchPanel());
            batchTabControl.TabPages.Add(mixedBatchTab);

            panel.Controls.Add(batchTabControl);
            return panel;
        }

        private Panel CreateBatchReadPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            // Title and description
            var titleLabel = new Label
            {
                Text = "🚀 Batch Read Operations - 3-10x Faster Than Individual Reads!",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(34, 197, 94),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            // Setup instructions panel
            var setupPanel = new Panel
            {
                Location = new Point(10, 35),
                Size = new Size(820, 80),
                BorderStyle = BorderStyle.FixedSingle,
                BackColor = Color.FromArgb(240, 245, 255)
            };
            
            var setupLabel = new Label
            {
                Text = "📋 Setup Instructions:",
                Location = new Point(5, 5),
                Font = new Font(this.Font, FontStyle.Bold),
                AutoSize = true
            };
            setupPanel.Controls.Add(setupLabel);
            
            var instructionText = new Label
            {
                Text = "1. Create test tags in your PLC: TestTag (BOOL), TestBool (BOOL), TestInt (DINT), TestReal (REAL), TestString (STRING)\n" +
                       "2. Or modify the tag names below to match existing tags in your PLC\n" +
                       "✅ Full STRING support available! Supports all Allen-Bradley data types including proper AB STRING format.",
                Location = new Point(5, 25),
                Size = new Size(800, 50),
                ForeColor = Color.FromArgb(75, 85, 99)
            };
            setupPanel.Controls.Add(instructionText);
            panel.Controls.Add(setupPanel);

            var descLabel = new Label
            {
                Text = "Enter multiple tag names (one per line) to read them all in a single optimized operation:",
                Location = new Point(10, 125),
                AutoSize = true
            };
            panel.Controls.Add(descLabel);

            // Input area
            var tagListTextBox = new TextBox
            {
                Name = "batchReadTagsTextBox",
                Location = new Point(10, 150),
                Size = new Size(300, 150),
                Multiline = true,
                ScrollBars = ScrollBars.Vertical,
                Text = "TestTag\nTestBool\nTestInt\nTestReal\nTestString\nTestString1"
            };
            panel.Controls.Add(tagListTextBox);

            // Execute button
            var executeButton = new Button
            {
                Name = "batchReadButton",
                Text = "🚀 Execute Batch Read",
                Location = new Point(10, 310),
                Size = new Size(150, 30),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            executeButton.Click += BatchReadButton_Click;
            panel.Controls.Add(executeButton);

            // Performance metrics
            var performanceLabel = new Label
            {
                Name = "batchReadPerformanceLabel",
                Text = "⏱️ Performance: Click execute to see timing",
                Location = new Point(170, 315),
                AutoSize = true
            };
            panel.Controls.Add(performanceLabel);

            // Results area
            var resultsLabel = new Label
            {
                Text = "📊 Results:",
                Location = new Point(330, 150),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            panel.Controls.Add(resultsLabel);

            var resultsListView = new ListView
            {
                Name = "batchReadResultsListView",
                Location = new Point(330, 175),
                Size = new Size(500, 150),
                View = View.Details,
                FullRowSelect = true,
                GridLines = true
            };
            resultsListView.Columns.Add("Tag Name", 150);
            resultsListView.Columns.Add("Value", 150);
            resultsListView.Columns.Add("Type", 80);
            resultsListView.Columns.Add("Status", 120);
            panel.Controls.Add(resultsListView);

            return panel;
        }

        private Panel CreateBatchWritePanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            // Title and description
            var titleLabel = new Label
            {
                Text = "✏️ Batch Write Operations - Atomic Multi-Tag Updates!",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(249, 115, 22),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            var descLabel = new Label
            {
                Text = "Enter tag=value pairs (one per line) to write them all in coordinated batches:",
                Location = new Point(10, 35),
                AutoSize = true
            };
            panel.Controls.Add(descLabel);

            // Input area
            var tagValueTextBox = new TextBox
            {
                Name = "batchWriteTagsTextBox",
                Location = new Point(10, 60),
                Size = new Size(300, 150),
                Multiline = true,
                ScrollBars = ScrollBars.Vertical,
                Text = "TestTag=true\nTestBool=false\nTestInt=999\nTestReal=88.8\nTestString=Batch Write Test\nTestString1=Production Status"
            };
            panel.Controls.Add(tagValueTextBox);

            // Execute button
            var executeButton = new Button
            {
                Name = "batchWriteButton",
                Text = "✏️ Execute Batch Write",
                Location = new Point(10, 220),
                Size = new Size(150, 30),
                BackColor = Color.FromArgb(249, 115, 22),
                ForeColor = Color.White,
                Enabled = false
            };
            executeButton.Click += BatchWriteButton_Click;
            panel.Controls.Add(executeButton);

            // Performance metrics
            var performanceLabel = new Label
            {
                Name = "batchWritePerformanceLabel",
                Text = "⏱️ Performance: Click execute to see timing",
                Location = new Point(170, 225),
                AutoSize = true
            };
            panel.Controls.Add(performanceLabel);

            // Results area
            var resultsLabel = new Label
            {
                Text = "📝 Results:",
                Location = new Point(330, 60),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            panel.Controls.Add(resultsLabel);

            var resultsListView = new ListView
            {
                Name = "batchWriteResultsListView",
                Location = new Point(330, 85),
                Size = new Size(500, 150),
                View = View.Details,
                FullRowSelect = true,
                GridLines = true
            };
            resultsListView.Columns.Add("Tag Name", 150);
            resultsListView.Columns.Add("Value", 100);
            resultsListView.Columns.Add("Type", 80);
            resultsListView.Columns.Add("Status", 170);
            panel.Controls.Add(resultsListView);

            return panel;
        }

        private Panel CreateMixedBatchPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            // Title and description
            var titleLabel = new Label
            {
                Text = "🔄 Mixed Batch Operations - Coordinated Read & Write!",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(147, 51, 234),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            var descLabel = new Label
            {
                Text = "Combine reads and writes in a single operation. Use 'READ:TagName' or 'WRITE:TagName=Value':",
                Location = new Point(10, 35),
                AutoSize = true
            };
            panel.Controls.Add(descLabel);

            // Input area
            var operationsTextBox = new TextBox
            {
                Name = "mixedBatchOperationsTextBox",
                Location = new Point(10, 60),
                Size = new Size(300, 150),
                Multiline = true,
                ScrollBars = ScrollBars.Vertical,
                Text = "READ:TestTag\nREAD:TestBool\nWRITE:TestInt=777\nWRITE:TestReal=99.9"
            };
            panel.Controls.Add(operationsTextBox);

            // Execute button
            var executeButton = new Button
            {
                Name = "mixedBatchButton",
                Text = "🔄 Execute Mixed Batch",
                Location = new Point(10, 220),
                Size = new Size(150, 30),
                BackColor = Color.FromArgb(147, 51, 234),
                ForeColor = Color.White,
                Enabled = false
            };
            executeButton.Click += MixedBatchButton_Click;
            panel.Controls.Add(executeButton);

            // Performance metrics
            var performanceLabel = new Label
            {
                Name = "mixedBatchPerformanceLabel",
                Text = "⏱️ Performance: Click execute to see timing",
                Location = new Point(170, 225),
                AutoSize = true
            };
            panel.Controls.Add(performanceLabel);

            // Results area
            var resultsLabel = new Label
            {
                Text = "🔄 Results:",
                Location = new Point(330, 60),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            panel.Controls.Add(resultsLabel);

            var resultsListView = new ListView
            {
                Name = "mixedBatchResultsListView",
                Location = new Point(330, 85),
                Size = new Size(500, 150),
                View = View.Details,
                FullRowSelect = true,
                GridLines = true
            };
            resultsListView.Columns.Add("Operation", 80);
            resultsListView.Columns.Add("Tag Name", 120);
            resultsListView.Columns.Add("Value", 100);
            resultsListView.Columns.Add("Time (μs)", 80);
            resultsListView.Columns.Add("Status", 120);
            panel.Controls.Add(resultsListView);

            return panel;
        }

        private Panel CreatePerformancePanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            // Title
            var titleLabel = new Label
            {
                Text = "📊 Performance Comparison: Individual vs Batch Operations",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(59, 130, 246),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            // Test configuration
            var configLabel = new Label
            {
                Text = "Test Configuration:",
                Location = new Point(10, 40),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            panel.Controls.Add(configLabel);

            var tagCountLabel = new Label
            {
                Text = "Number of tags:",
                Location = new Point(10, 65),
                AutoSize = true
            };
            panel.Controls.Add(tagCountLabel);

            var tagCountNumeric = new NumericUpDown
            {
                Name = "tagCountNumeric",
                Location = new Point(120, 62),
                Size = new Size(60, 23),
                Minimum = 1,
                Maximum = 50,
                Value = 5
            };
            panel.Controls.Add(tagCountNumeric);

            var testTypeLabel = new Label
            {
                Text = "Test type:",
                Location = new Point(200, 65),
                AutoSize = true
            };
            panel.Controls.Add(testTypeLabel);

            var testTypeCombo = new ComboBox
            {
                Name = "testTypeCombo",
                Location = new Point(270, 62),
                Size = new Size(100, 23),
                DropDownStyle = ComboBoxStyle.DropDownList
            };
            testTypeCombo.Items.AddRange(new[] { "Read Only", "Write Only", "Mixed" });
            testTypeCombo.SelectedIndex = 0;
            panel.Controls.Add(testTypeCombo);

            // Run benchmark button
            var benchmarkButton = new Button
            {
                Name = "benchmarkButton",
                Text = "🚀 Run Performance Test",
                Location = new Point(10, 95),
                Size = new Size(150, 30),
                BackColor = Color.FromArgb(59, 130, 246),
                ForeColor = Color.White,
                Enabled = false
            };
            benchmarkButton.Click += BenchmarkButton_Click;
            panel.Controls.Add(benchmarkButton);

            // Results display
            var resultsGroupBox = new GroupBox
            {
                Text = "📊 Performance Results",
                Location = new Point(10, 140),
                Size = new Size(800, 250)
            };

            var individualLabel = new Label
            {
                Name = "individualPerformanceLabel",
                Text = "🐌 Individual Operations: Not tested yet",
                Location = new Point(10, 25),
                AutoSize = true
            };
            resultsGroupBox.Controls.Add(individualLabel);

            var batchLabel = new Label
            {
                Name = "batchPerformanceLabel",
                Text = "🚀 Batch Operations: Not tested yet",
                Location = new Point(10, 50),
                AutoSize = true
            };
            resultsGroupBox.Controls.Add(batchLabel);

            var improvementLabel = new Label
            {
                Name = "improvementLabel",
                Text = "📈 Performance Improvement: N/A",
                Location = new Point(10, 75),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(34, 197, 94)
            };
            resultsGroupBox.Controls.Add(improvementLabel);

            var networlLabel = new Label
            {
                Name = "networkEfficiencyLabel", 
                Text = "📡 Network Efficiency: N/A",
                Location = new Point(10, 100),
                AutoSize = true
            };
            resultsGroupBox.Controls.Add(networlLabel);

            // Performance chart (simplified text-based)
            var chartLabel = new Label
            {
                Text = "📊 Performance Chart:",
                Location = new Point(10, 130),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            resultsGroupBox.Controls.Add(chartLabel);

            var chartTextBox = new TextBox
            {
                Name = "performanceChartTextBox",
                Location = new Point(10, 155),
                Size = new Size(770, 80),
                Multiline = true,
                ReadOnly = true,
                ScrollBars = ScrollBars.Vertical,
                Text = "Run a performance test to see detailed timing comparison..."
            };
            resultsGroupBox.Controls.Add(chartTextBox);

            panel.Controls.Add(resultsGroupBox);

            return panel;
        }

        private Panel CreateBatchConfigPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            // Title
            var titleLabel = new Label
            {
                Text = "⚙️ Batch Operation Configuration - Optimize for Your PLC",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(147, 51, 234),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            // Current config display
            var currentConfigGroupBox = new GroupBox
            {
                Text = "📋 Current Configuration",
                Location = new Point(10, 40),
                Size = new Size(400, 200)
            };

            var currentConfigLabel = new Label
            {
                Name = "currentConfigLabel",
                Text = "Loading configuration...",
                Location = new Point(10, 25),
                Size = new Size(380, 160),
                AutoSize = false
            };
            currentConfigGroupBox.Controls.Add(currentConfigLabel);

            panel.Controls.Add(currentConfigGroupBox);

            // Preset configurations
            var presetsGroupBox = new GroupBox
            {
                Text = "🎯 Preset Configurations",
                Location = new Point(430, 40),
                Size = new Size(300, 200)
            };

            var defaultButton = new Button
            {
                Name = "defaultConfigButton",
                Text = "📊 Default Config",
                Location = new Point(10, 25),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(59, 130, 246),
                ForeColor = Color.White,
                Enabled = false
            };
            defaultButton.Click += DefaultConfigButton_Click;
            presetsGroupBox.Controls.Add(defaultButton);

            var highPerfButton = new Button
            {
                Name = "highPerfConfigButton",
                Text = "🚀 High Performance",
                Location = new Point(10, 65),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            highPerfButton.Click += HighPerfConfigButton_Click;
            presetsGroupBox.Controls.Add(highPerfButton);

            var conservativeButton = new Button
            {
                Name = "conservativeConfigButton",
                Text = "🛡️ Conservative",
                Location = new Point(10, 105),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(249, 115, 22),
                ForeColor = Color.White,
                Enabled = false
            };
            conservativeButton.Click += ConservativeConfigButton_Click;
            presetsGroupBox.Controls.Add(conservativeButton);

            // Preset descriptions
            var presetDescTextBox = new TextBox
            {
                Name = "presetDescTextBox",
                Location = new Point(140, 25),
                Size = new Size(150, 140),
                Multiline = true,
                ReadOnly = true,
                ScrollBars = ScrollBars.Vertical,
                Text = "Default: 20 ops/packet, 504 bytes\n\nHigh Performance: 50 ops/packet, 4000 bytes\n\nConservative: 10 ops/packet, 504 bytes"
            };
            presetsGroupBox.Controls.Add(presetDescTextBox);

            panel.Controls.Add(presetsGroupBox);

            // Custom configuration
            var customConfigGroupBox = new GroupBox
            {
                Text = "🔧 Custom Configuration",
                Location = new Point(10, 260),
                Size = new Size(720, 200)
            };

            // Max operations per packet
            var maxOpsLabel = new Label
            {
                Text = "Max operations per packet:",
                Location = new Point(10, 25),
                AutoSize = true
            };
            customConfigGroupBox.Controls.Add(maxOpsLabel);

            var maxOpsNumeric = new NumericUpDown
            {
                Name = "maxOpsNumeric",
                Location = new Point(200, 22),
                Size = new Size(60, 23),
                Minimum = 1,
                Maximum = 100,
                Value = 20
            };
            customConfigGroupBox.Controls.Add(maxOpsNumeric);

            // Max packet size
            var maxPacketLabel = new Label
            {
                Text = "Max packet size (bytes):",
                Location = new Point(10, 55),
                AutoSize = true
            };
            customConfigGroupBox.Controls.Add(maxPacketLabel);

            var maxPacketNumeric = new NumericUpDown
            {
                Name = "maxPacketNumeric",
                Location = new Point(200, 52),
                Size = new Size(80, 23),
                Minimum = 200,
                Maximum = 8000,
                Value = 504,
                Increment = 100
            };
            customConfigGroupBox.Controls.Add(maxPacketNumeric);

            // Timeout
            var timeoutLabel = new Label
            {
                Text = "Packet timeout (ms):",
                Location = new Point(10, 85),
                AutoSize = true
            };
            customConfigGroupBox.Controls.Add(timeoutLabel);

            var timeoutNumeric = new NumericUpDown
            {
                Name = "timeoutNumeric",
                Location = new Point(200, 82),
                Size = new Size(80, 23),
                Minimum = 500,
                Maximum = 30000,
                Value = 3000,
                Increment = 500
            };
            customConfigGroupBox.Controls.Add(timeoutNumeric);

            // Continue on error
            var continueOnErrorCheck = new CheckBox
            {
                Name = "continueOnErrorCheck",
                Text = "Continue processing on individual operation errors",
                Location = new Point(10, 115),
                AutoSize = true,
                Checked = true
            };
            customConfigGroupBox.Controls.Add(continueOnErrorCheck);

            // Optimize packing
            var optimizePackingCheck = new CheckBox
            {
                Name = "optimizePackingCheck",
                Text = "Optimize packet packing (group similar operations)",
                Location = new Point(10, 140),
                AutoSize = true,
                Checked = true
            };
            customConfigGroupBox.Controls.Add(optimizePackingCheck);

            // Apply custom config button
            var applyCustomButton = new Button
            {
                Name = "applyCustomConfigButton",
                Text = "🔧 Apply Custom Config",
                Location = new Point(300, 85),
                Size = new Size(140, 30),
                BackColor = Color.FromArgb(147, 51, 234),
                ForeColor = Color.White,
                Enabled = false
            };
            applyCustomButton.Click += ApplyCustomConfigButton_Click;
            customConfigGroupBox.Controls.Add(applyCustomButton);

            panel.Controls.Add(customConfigGroupBox);

            return panel;
        }

        private Panel CreateLogPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill };

            var logLabel = new Label
            {
                Text = "📝 Activity Log:",
                Location = new Point(10, 10),
                AutoSize = true,
                Font = new Font(this.Font, FontStyle.Bold)
            };
            panel.Controls.Add(logLabel);

            var logTextBox = new TextBox
            {
                Name = "logTextBox",
                Location = new Point(10, 35),
                Size = new Size(panel.Width - 20, panel.Height - 75),
                Anchor = AnchorStyles.Top | AnchorStyles.Bottom | AnchorStyles.Left | AnchorStyles.Right,
                Multiline = true,
                ScrollBars = ScrollBars.Vertical,
                ReadOnly = true,
                BackColor = Color.Black,
                ForeColor = Color.LimeGreen,
                Font = new Font("Consolas", 9)
            };
            panel.Controls.Add(logTextBox);

            var clearLogButton = new Button
            {
                Name = "clearLogButton",
                Text = "Clear Log",
                Location = new Point(10, panel.Height - 35),
                Size = new Size(80, 25),
                Anchor = AnchorStyles.Bottom | AnchorStyles.Left
            };
            clearLogButton.Click += (s, e) => logTextBox.Clear();
            panel.Controls.Add(clearLogButton);

            return panel;
        }

        private void SetupTimers()
        {
            _connectionMonitorTimer = new System.Windows.Forms.Timer();
            _connectionMonitorTimer.Interval = 10000; // 10 seconds
            _connectionMonitorTimer.Tick += ConnectionMonitorTimer_Tick;
            _connectionMonitorTimer.Start();
        }

        private void UpdateConnectionStatus()
        {
            var statusLabel = (Label)Controls.Find("statusLabel", true)[0];
            var sessionLabel = (Label)Controls.Find("sessionLabel", true)[0];
            var connectButton = (Button)Controls.Find("connectButton", true)[0];
            var disconnectButton = (Button)Controls.Find("disconnectButton", true)[0];
            var benchmarkButton = (Button)Controls.Find("benchmarkButton", true)[0];
            var discoverButton = (Button)Controls.Find("discoverButton", true)[0];
            var readButton = (Button)Controls.Find("readButton", true)[0];
            var writeButton = (Button)Controls.Find("writeButton", true)[0];

            // Batch operation buttons
            var batchReadButton = Controls.Find("batchReadButton", true).FirstOrDefault() as Button;
            var batchWriteButton = Controls.Find("batchWriteButton", true).FirstOrDefault() as Button;
            var mixedBatchButton = Controls.Find("mixedBatchButton", true).FirstOrDefault() as Button;
            var defaultConfigButton = Controls.Find("defaultConfigButton", true).FirstOrDefault() as Button;
            var highPerfConfigButton = Controls.Find("highPerfConfigButton", true).FirstOrDefault() as Button;
            var conservativeConfigButton = Controls.Find("conservativeConfigButton", true).FirstOrDefault() as Button;
            var applyCustomConfigButton = Controls.Find("applyCustomConfigButton", true).FirstOrDefault() as Button;

            if (_isConnected)
            {
                statusLabel.Text = "Connected";
                statusLabel.ForeColor = Color.FromArgb(16, 185, 129);
                sessionLabel.Text = $"Session: 0x{_plcClient?.ClientId:X8}";
                connectButton.Enabled = false;
                disconnectButton.Enabled = true;
                benchmarkButton.Enabled = true;
                discoverButton.Enabled = true;
                readButton.Enabled = true;
                writeButton.Enabled = true;

                // Enable batch operation buttons
                if (batchReadButton != null) batchReadButton.Enabled = true;
                if (batchWriteButton != null) batchWriteButton.Enabled = true;
                if (mixedBatchButton != null) mixedBatchButton.Enabled = true;
                if (defaultConfigButton != null) defaultConfigButton.Enabled = true;
                if (highPerfConfigButton != null) highPerfConfigButton.Enabled = true;
                if (conservativeConfigButton != null) conservativeConfigButton.Enabled = true;
                if (applyCustomConfigButton != null) applyCustomConfigButton.Enabled = true;

                // Update current config display
                UpdateCurrentConfigDisplay();
            }
            else
            {
                statusLabel.Text = "Disconnected";
                statusLabel.ForeColor = Color.FromArgb(239, 68, 68);
                sessionLabel.Text = "Session: None";
                connectButton.Enabled = true;
                disconnectButton.Enabled = false;
                benchmarkButton.Enabled = false;
                discoverButton.Enabled = false;
                readButton.Enabled = false;
                writeButton.Enabled = false;

                // Disable batch operation buttons
                if (batchReadButton != null) batchReadButton.Enabled = false;
                if (batchWriteButton != null) batchWriteButton.Enabled = false;
                if (mixedBatchButton != null) mixedBatchButton.Enabled = false;
                if (defaultConfigButton != null) defaultConfigButton.Enabled = false;
                if (highPerfConfigButton != null) highPerfConfigButton.Enabled = false;
                if (conservativeConfigButton != null) conservativeConfigButton.Enabled = false;
                if (applyCustomConfigButton != null) applyCustomConfigButton.Enabled = false;
            }
        }

        private void UpdateCurrentConfigDisplay()
        {
            if (_plcClient == null) return;

            var currentConfigLabel = Controls.Find("currentConfigLabel", true).FirstOrDefault() as Label;
            if (currentConfigLabel == null) return;

            try
            {
                var config = _plcClient.GetBatchConfig();
                currentConfigLabel.Text = $"📊 Max Operations per Packet: {config.MaxOperationsPerPacket}\n" +
                                        $"📦 Max Packet Size: {config.MaxPacketSize} bytes\n" +
                                        $"⏱️ Packet Timeout: {config.PacketTimeoutMs} ms\n" +
                                        $"🔄 Continue on Error: {config.ContinueOnError}\n" +
                                        $"🎯 Optimize Packing: {config.OptimizePacketPacking}";
            }
            catch (Exception ex)
            {
                currentConfigLabel.Text = $"Error loading config: {ex.Message}";
            }
        }

        private void Log(string message)
        {
            var logTextBox = (TextBox)Controls.Find("logTextBox", true)[0];
            var timestamp = DateTime.Now.ToString("HH:mm:ss");
            logTextBox.AppendText($"[{timestamp}] {message}{Environment.NewLine}");
            logTextBox.ScrollToCaret();
        }

        private void ConnectButton_Click(object? sender, EventArgs e)
        {
            var plcAddressTextBox = (TextBox)Controls.Find("plcAddressTextBox", true)[0];
            var address = plcAddressTextBox.Text.Trim();

            if (string.IsNullOrEmpty(address))
            {
                Log("❌ Please enter a PLC address");
                return;
            }

            try
            {
                Log("🔌 Connecting to PLC...");
                _plcClient = new EtherNetIpClient();
                _isConnected = _plcClient.Connect(address);
                _currentAddress = address;

                if (_isConnected)
                {
                    Log($"✅ Connected! Session ID: 0x{_plcClient.ClientId:X8}");
                    UpdateConnectionStatus();
                    _ = InitializeTags();
                }
                else
                {
                    Log("❌ Connection failed");
                    _isConnected = false;
                    UpdateConnectionStatus();
                }
            }
            catch (Exception ex)
            {
                Log($"❌ Connection error: {ex.Message}");
                _isConnected = false;
                UpdateConnectionStatus();
            }
        }

        private void DisconnectButton_Click(object? sender, EventArgs e)
        {
            try
            {
                if (_plcClient != null)
                {
                    _plcClient.Dispose();
                }
                _isConnected = false;
                _currentAddress = string.Empty;
                UpdateConnectionStatus();
                Log("📤 Disconnected from PLC");
            }
            catch (Exception ex)
            {
                Log($"⚠️ Disconnect error: {ex.Message}");
            }
        }

        private async Task InitializeTags()
        {
            if (!_isConnected || _plcClient == null) return;

            try
            {
                Log("🔍 Initializing test tags...");

                // Updated test tags to include STRING examples (now fully supported!)
                var testTags = new(string name, string type, object value)[]
                {
                    ("TestTag", "BOOL", true),
                    ("TestBool", "BOOL", false),
                    ("TestInt", "DINT", 42),
                    ("TestReal", "REAL", 123.45f),
                    ("TestString", "STRING", "WinForms Demo"),
                    ("TestString1", "STRING", "Hello World!"),
                    ("TestString2", "STRING", "Production Line Status")
                };

                foreach (var (name, type, value) in testTags)
                {
                    try
                    {
                        await Task.Run(() =>
                        {
                            switch (type)
                            {
                                case "BOOL":
                                    _plcClient.WriteBool(name, (bool)value);
                                    break;
                                case "DINT":
                                    _plcClient.WriteDint(name, (int)value);
                                    break;
                                case "REAL":
                                    _plcClient.WriteReal(name, (float)value);
                                    break;
                                case "STRING":
                                    _plcClient.WriteString(name, (string)value);
                                    break;
                            }
                        });
                        
                        _tags[name] = new TagInfo 
                        { 
                            Name = name, 
                            Type = type, 
                            Value = value, 
                            Updated = DateTime.Now 
                        };
                        
                        Log($"✅ Initialized {type} tag: {name} = {value}");
                    }
                    catch (Exception ex)
                    {
                        Log($"⚠️ Could not initialize tag {name}: {ex.Message}");
                        Log($"💡 Tag {name} may not exist in PLC - you can create it manually");
                    }
                }

                Log("✅ Test tag initialization complete");
                Log("🚀 STRING operations fully supported with proper Allen-Bradley format!");
                Log("🚀 Ready for batch operations testing including STRING read/write!");
            }
            catch (Exception ex)
            {
                Log($"❌ Error during tag initialization: {ex.Message}");
            }
        }

        private void ConnectionMonitorTimer_Tick(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null || _isReconnecting) return;

            try
            {
                // Use a more lightweight health check instead of reading a tag
                if (!_plcClient.CheckHealth())
                {
                    throw new Exception("Health check failed");
                }
            }
            catch
            {
                Log("⚠️ Connection lost");
                _isConnected = false;
                UpdateConnectionStatus();
                AttemptReconnect();
            }
        }

        private async void AttemptReconnect()
        {
            if (_isReconnecting) return;
            _isReconnecting = true;

            try
            {
                // First try to disconnect cleanly
                if (_plcClient != null)
                {
                    try
                    {
                        _plcClient.Dispose();
                    }
                    catch (Exception ex)
                    {
                        Log($"Warning during disconnect: {ex.Message}");
                    }
                }

                // Exponential backoff for retries
                int delay = RETRY_DELAY * (int)Math.Pow(2, _retryCount);
                await Task.Delay(delay);

                _plcClient = new EtherNetIpClient();
                _isConnected = _plcClient.Connect(_currentAddress);

                if (_isConnected)
                {
                    Log("✅ Reconnected successfully");
                    _retryCount = 0;
                    UpdateConnectionStatus();
                    await InitializeTags();
                }
                else
                {
                    throw new Exception("Reconnection failed");
                }
            }
            catch (Exception ex)
            {
                Log($"❌ Reconnection failed: {ex.Message}");
                _isConnected = false;
                UpdateConnectionStatus();
                
                _retryCount++;
                if (_retryCount >= MAX_RETRIES)
                {
                    Log("❌ Max retry attempts reached. Please try connecting manually.");
                    _retryCount = 0;
                }
            }
            finally
            {
                _isReconnecting = false;
            }
        }

        private void BenchmarkButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var tagCountNumeric = Controls.Find("tagCountNumeric", true).FirstOrDefault() as NumericUpDown;
            var testTypeCombo = Controls.Find("testTypeCombo", true).FirstOrDefault() as ComboBox;
            var individualLabel = Controls.Find("individualPerformanceLabel", true).FirstOrDefault() as Label;
            var batchLabel = Controls.Find("batchPerformanceLabel", true).FirstOrDefault() as Label;
            var improvementLabel = Controls.Find("improvementLabel", true).FirstOrDefault() as Label;
            var networkLabel = Controls.Find("networkEfficiencyLabel", true).FirstOrDefault() as Label;
            var chartTextBox = Controls.Find("performanceChartTextBox", true).FirstOrDefault() as TextBox;

            if (tagCountNumeric == null || testTypeCombo == null) return;

            var tagCount = (int)tagCountNumeric.Value;
            var testType = testTypeCombo.SelectedItem?.ToString() ?? "Read Only";

            try
            {
                Log($"📊 Starting performance benchmark: {tagCount} tags, {testType}");

                // Generate test tag names
                var testTags = Enumerable.Range(1, tagCount)
                    .Select(i => $"TestTag_{i}")
                    .ToArray();

                // Ensure test tags exist for read tests
                if (testType != "Write Only")
                {
                    Log("📝 Preparing test tags...");
                    foreach (var tag in testTags.Take(Math.Min(5, tagCount))) // Only create a few test tags
                    {
                        try
                        {
                            _plcClient.WriteBool(tag, true);
                        }
                        catch
                        {
                            // Tag might not exist, that's ok for demo
                        }
                    }
                }

                // Test Individual Operations
                Log("🐌 Testing individual operations...");
                var individualStopwatch = Stopwatch.StartNew();
                int individualSuccessCount = 0;

                switch (testType)
                {
                    case "Read Only":
                        foreach (var tag in testTags)
                        {
                            try
                            {
                                _plcClient.ReadBool(tag);
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;

                    case "Write Only":
                        foreach (var tag in testTags)
                        {
                            try
                            {
                                _plcClient.WriteBool(tag, true);
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;

                    case "Mixed":
                        for (int i = 0; i < testTags.Length; i++)
                        {
                            try
                            {
                                if (i % 2 == 0)
                                {
                                    _plcClient.ReadBool(testTags[i]);
                                }
                                else
                                {
                                    _plcClient.WriteBool(testTags[i], true);
                                }
                                individualSuccessCount++;
                            }
                            catch { }
                        }
                        break;
                }

                individualStopwatch.Stop();
                var individualTime = individualStopwatch.ElapsedMilliseconds;

                // Test Batch Operations
                Log("🚀 Testing batch operations...");
                var batchStopwatch = Stopwatch.StartNew();
                int batchSuccessCount = 0;

                switch (testType)
                {
                    case "Read Only":
                        try
                        {
                            var results = _plcClient.ReadTagsBatch(testTags);
                            batchSuccessCount = results.Count(r => r.Value.Success);
                        }
                        catch { }
                        break;

                    case "Write Only":
                        try
                        {
                            var tagValues = testTags.ToDictionary(tag => tag, tag => (object)true);
                            var results = _plcClient.WriteTagsBatch(tagValues);
                            batchSuccessCount = results.Count(r => r.Value.Success);
                        }
                        catch { }
                        break;

                    case "Mixed":
                        try
                        {
                            var operations = new List<BatchOperation>();
                            for (int i = 0; i < testTags.Length; i++)
                            {
                                if (i % 2 == 0)
                                {
                                    operations.Add(BatchOperation.Read(testTags[i]));
                                }
                                else
                                {
                                    operations.Add(BatchOperation.Write(testTags[i], true));
                                }
                            }
                            var results = _plcClient.ExecuteBatch(operations.ToArray());
                            batchSuccessCount = results.Count(r => r.Success);
                        }
                        catch { }
                        break;
                }

                batchStopwatch.Stop();
                var batchTime = batchStopwatch.ElapsedMilliseconds;

                // Calculate performance metrics
                var speedup = batchTime > 0 ? (double)individualTime / batchTime : 0;
                var networkEfficiency = tagCount; // 1 packet vs N packets

                // Update UI
                if (individualLabel != null)
                {
                    individualLabel.Text = $"🐌 Individual Operations: {individualTime}ms total, {(double)individualTime / tagCount:F1}ms avg, {individualSuccessCount}/{tagCount} successful";
                }

                if (batchLabel != null)
                {
                    batchLabel.Text = $"🚀 Batch Operations: {batchTime}ms total, {(double)batchTime / tagCount:F1}ms avg, {batchSuccessCount}/{tagCount} successful";
                }

                if (improvementLabel != null)
                {
                    if (speedup > 0)
                    {
                        improvementLabel.Text = $"📈 Performance Improvement: {speedup:F1}x faster with batch operations!";
                        improvementLabel.ForeColor = speedup > 2 ? Color.FromArgb(34, 197, 94) : Color.FromArgb(249, 115, 22);
                    }
                    else
                    {
                        improvementLabel.Text = "📈 Performance Improvement: Unable to calculate";
                        improvementLabel.ForeColor = Color.FromArgb(107, 114, 128);
                    }
                }

                if (networkLabel != null)
                {
                    networkLabel.Text = $"📡 Network Efficiency: ~{networkEfficiency}x fewer packets (1 vs {tagCount})";
                }

                // Create performance chart
                if (chartTextBox != null)
                {
                    var chart = new System.Text.StringBuilder();
                    chart.AppendLine($"Performance Comparison Results ({testType}):");
                    chart.AppendLine($"{'=',-50}");
                    chart.AppendLine($"Test Configuration: {tagCount} tags");
                    chart.AppendLine();
                    chart.AppendLine($"Individual Operations:");
                    chart.AppendLine($"  Total Time: {individualTime}ms");
                    chart.AppendLine($"  Average per operation: {(double)individualTime / tagCount:F1}ms");
                    chart.AppendLine($"  Success rate: {(double)individualSuccessCount / tagCount * 100:F1}%");
                    chart.AppendLine($"  Network packets: ~{tagCount} (one per operation)");
                    chart.AppendLine();
                    chart.AppendLine($"Batch Operations:");
                    chart.AppendLine($"  Total Time: {batchTime}ms");
                    chart.AppendLine($"  Average per operation: {(double)batchTime / tagCount:F1}ms");
                    chart.AppendLine($"  Success rate: {(double)batchSuccessCount / tagCount * 100:F1}%");
                    chart.AppendLine($"  Network packets: ~1-3 (optimized batching)");
                    chart.AppendLine();
                    
                    if (speedup > 0)
                    {
                        chart.AppendLine($"Performance Improvement:");
                        chart.AppendLine($"  Speed: {speedup:F1}x faster");
                        chart.AppendLine($"  Time saved: {individualTime - batchTime}ms ({(1 - (double)batchTime / individualTime) * 100:F1}%)");
                        chart.AppendLine($"  Network efficiency: {networkEfficiency}x fewer packets");
                        
                        // Visual bar chart
                        chart.AppendLine();
                        chart.AppendLine("Visual Comparison:");
                        var maxBarLength = 40;
                        var individualBar = new string('█', Math.Min(maxBarLength, (int)(individualTime * maxBarLength / Math.Max(individualTime, batchTime))));
                        var batchBar = new string('█', Math.Min(maxBarLength, (int)(batchTime * maxBarLength / Math.Max(individualTime, batchTime))));
                        
                        chart.AppendLine($"Individual: {individualBar} {individualTime}ms");
                        chart.AppendLine($"Batch:      {batchBar} {batchTime}ms");
                    }

                    chartTextBox.Text = chart.ToString();
                }

                Log($"✅ Benchmark completed: Individual={individualTime}ms, Batch={batchTime}ms, Speedup={speedup:F1}x");
            }
            catch (Exception ex)
            {
                Log($"❌ Benchmark error: {ex.Message}");
                
                if (individualLabel != null) individualLabel.Text = "🐌 Individual Operations: Error occurred";
                if (batchLabel != null) batchLabel.Text = "🚀 Batch Operations: Error occurred";
                if (improvementLabel != null) improvementLabel.Text = "📈 Performance Improvement: Test failed";
                if (chartTextBox != null) chartTextBox.Text = $"Benchmark failed: {ex.Message}";
            }
        }

        private void DiscoverButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var discoverTextBox = (TextBox)Controls.Find("discoverTextBox", true)[0];
            var tagName = discoverTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name to discover");
                return;
            }

            try
            {
                Log($"🔍 Discovering tag: {tagName}");

                // Try to read the tag as different types - order matters for proper detection
                try
                {
                    var boolValue = _plcClient.ReadBool(tagName);
                    UpdateTagFields(tagName, "BOOL", boolValue.ToString());
                    Log($"✅ Discovered BOOL tag: {tagName} = {boolValue}");
                    return;
                }
                catch { }

                try
                {
                    var sintValue = _plcClient.ReadSint(tagName);
                    UpdateTagFields(tagName, "SINT", sintValue.ToString());
                    Log($"✅ Discovered SINT tag: {tagName} = {sintValue}");
                    return;
                }
                catch { }

                try
                {
                    var intValue = _plcClient.ReadInt(tagName);
                    UpdateTagFields(tagName, "INT", intValue.ToString());
                    Log($"✅ Discovered INT tag: {tagName} = {intValue}");
                    return;
                }
                catch { }

                try
                {
                    var dintValue = _plcClient.ReadDint(tagName);
                    UpdateTagFields(tagName, "DINT", dintValue.ToString());
                    Log($"✅ Discovered DINT tag: {tagName} = {dintValue}");
                    return;
                }
                catch { }

                try
                {
                    var lintValue = _plcClient.ReadLint(tagName);
                    UpdateTagFields(tagName, "LINT", lintValue.ToString());
                    Log($"✅ Discovered LINT tag: {tagName} = {lintValue}");
                    return;
                }
                catch { }

                try
                {
                    var usintValue = _plcClient.ReadUsint(tagName);
                    UpdateTagFields(tagName, "USINT", usintValue.ToString());
                    Log($"✅ Discovered USINT tag: {tagName} = {usintValue}");
                    return;
                }
                catch { }

                try
                {
                    var uintValue = _plcClient.ReadUint(tagName);
                    UpdateTagFields(tagName, "UINT", uintValue.ToString());
                    Log($"✅ Discovered UINT tag: {tagName} = {uintValue}");
                    return;
                }
                catch { }

                try
                {
                    var udintValue = _plcClient.ReadUdint(tagName);
                    UpdateTagFields(tagName, "UDINT", udintValue.ToString());
                    Log($"✅ Discovered UDINT tag: {tagName} = {udintValue}");
                    return;
                }
                catch { }

                try
                {
                    var ulintValue = _plcClient.ReadUlint(tagName);
                    UpdateTagFields(tagName, "ULINT", ulintValue.ToString());
                    Log($"✅ Discovered ULINT tag: {tagName} = {ulintValue}");
                    return;
                }
                catch { }

                try
                {
                    var realValue = _plcClient.ReadReal(tagName);
                    UpdateTagFields(tagName, "REAL", realValue.ToString());
                    Log($"✅ Discovered REAL tag: {tagName} = {realValue}");
                    return;
                }
                catch { }

                try
                {
                    var lrealValue = _plcClient.ReadLreal(tagName);
                    UpdateTagFields(tagName, "LREAL", lrealValue.ToString());
                    Log($"✅ Discovered LREAL tag: {tagName} = {lrealValue}");
                    return;
                }
                catch { }

                try
                {
                    var stringValue = _plcClient.ReadString(tagName);
                    UpdateTagFields(tagName, "STRING", stringValue);
                    Log($"✅ Discovered STRING tag: {tagName} = '{stringValue}'");
                    return;
                }
                catch { }

                try
                {
                    var udtValue = _plcClient.ReadUdt(tagName);
                    var memberCount = udtValue.UdtMembers?.Count ?? 0;
                    UpdateTagFields(tagName, "UDT", $"UDT with {memberCount} members");
                    Log($"✅ Discovered UDT tag: {tagName} with {memberCount} members");
                    return;
                }
                catch { }

                Log($"❌ Could not determine type for tag: {tagName}");
            }
            catch (Exception ex)
            {
                Log($"❌ Discovery error: {ex.Message}");
            }
        }

        private void UpdateTagFields(string tagName, string type, string value)
        {
            var tagNameTextBox = (TextBox)Controls.Find("tagNameTextBox", true)[0];
            var dataTypeComboBox = (ComboBox)Controls.Find("dataTypeComboBox", true)[0];
            var tagValueTextBox = (TextBox)Controls.Find("tagValueTextBox", true)[0];

            tagNameTextBox.Text = tagName;
            dataTypeComboBox.SelectedItem = type;
            tagValueTextBox.Text = value;
        }

        private void ReadButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var tagNameTextBox = (TextBox)Controls.Find("tagNameTextBox", true)[0];
            var tagValueTextBox = (TextBox)Controls.Find("tagValueTextBox", true)[0];
            var tagName = tagNameTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name");
                return;
            }

            try
            {
                Log($"📖 Reading tag: {tagName}");

                // Try to read the tag as different types
                try
                {
                    var boolValue = _plcClient.ReadBool(tagName);
                    tagValueTextBox.Text = boolValue.ToString();
                    Log($"✅ Read BOOL tag: {tagName} = {boolValue}");
                    return;
                }
                catch { }

                try
                {
                    var dintValue = _plcClient.ReadDint(tagName);
                    tagValueTextBox.Text = dintValue.ToString();
                    Log($"✅ Read DINT tag: {tagName} = {dintValue}");
                    return;
                }
                catch { }

                try
                {
                    var realValue = _plcClient.ReadReal(tagName);
                    tagValueTextBox.Text = realValue.ToString();
                    Log($"✅ Read REAL tag: {tagName} = {realValue}");
                    return;
                }
                catch { }

                try
                {
                    var stringValue = _plcClient.ReadString(tagName);
                    tagValueTextBox.Text = stringValue;
                    Log($"✅ Read STRING tag: {tagName} = {stringValue}");
                    return;
                }
                catch { }

                Log($"❌ Could not read tag: {tagName}");
            }
            catch (Exception ex)
            {
                Log($"❌ Read error: {ex.Message}");
            }
        }

        private void WriteButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var tagNameTextBox = (TextBox)Controls.Find("tagNameTextBox", true)[0];
            var dataTypeComboBox = (ComboBox)Controls.Find("dataTypeComboBox", true)[0];
            var tagValueTextBox = (TextBox)Controls.Find("tagValueTextBox", true)[0];
            var tagName = tagNameTextBox.Text.Trim();
            var type = dataTypeComboBox.SelectedItem?.ToString() ?? string.Empty;
            var value = tagValueTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name");
                return;
            }

            try
            {
                Log($"✏️ Writing tag: {tagName}");

                switch (type)
                {
                    case "BOOL":
                        if (bool.TryParse(value, out bool boolValue))
                        {
                            _plcClient.WriteBool(tagName, boolValue);
                            Log($"✅ Wrote BOOL: {boolValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid boolean value");
                        }
                        break;

                    case "SINT":
                        if (sbyte.TryParse(value, out sbyte sintValue))
                        {
                            _plcClient.WriteSint(tagName, sintValue);
                            Log($"✅ Wrote SINT: {sintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid SINT value (-128 to 127)");
                        }
                        break;

                    case "INT":
                        if (short.TryParse(value, out short intValue))
                        {
                            _plcClient.WriteInt(tagName, intValue);
                            Log($"✅ Wrote INT: {intValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid INT value (-32,768 to 32,767)");
                        }
                        break;

                    case "DINT":
                        if (int.TryParse(value, out int dintValue))
                        {
                            _plcClient.WriteDint(tagName, dintValue);
                            Log($"✅ Wrote DINT: {dintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid DINT value");
                        }
                        break;

                    case "LINT":
                        if (long.TryParse(value, out long lintValue))
                        {
                            _plcClient.WriteLint(tagName, lintValue);
                            Log($"✅ Wrote LINT: {lintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid LINT value");
                        }
                        break;

                    case "USINT":
                        if (byte.TryParse(value, out byte usintValue))
                        {
                            _plcClient.WriteUsint(tagName, usintValue);
                            Log($"✅ Wrote USINT: {usintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid USINT value (0 to 255)");
                        }
                        break;

                    case "UINT":
                        if (ushort.TryParse(value, out ushort uintValue))
                        {
                            _plcClient.WriteUint(tagName, uintValue);
                            Log($"✅ Wrote UINT: {uintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid UINT value (0 to 65,535)");
                        }
                        break;

                    case "UDINT":
                        if (uint.TryParse(value, out uint udintValue))
                        {
                            _plcClient.WriteUdint(tagName, udintValue);
                            Log($"✅ Wrote UDINT: {udintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid UDINT value");
                        }
                        break;

                    case "ULINT":
                        if (ulong.TryParse(value, out ulong ulintValue))
                        {
                            _plcClient.WriteUlint(tagName, ulintValue);
                            Log($"✅ Wrote ULINT: {ulintValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid ULINT value");
                        }
                        break;

                    case "REAL":
                        if (float.TryParse(value, out float realValue))
                        {
                            _plcClient.WriteReal(tagName, realValue);
                            Log($"✅ Wrote REAL: {realValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid REAL value");
                        }
                        break;

                    case "LREAL":
                        if (double.TryParse(value, out double lrealValue))
                        {
                            _plcClient.WriteLreal(tagName, lrealValue);
                            Log($"✅ Wrote LREAL: {lrealValue} to {tagName}");
                        }
                        else
                        {
                            Log("❌ Invalid LREAL value");
                        }
                        break;

                    case "STRING":
                        _plcClient.WriteString(tagName, value);
                        Log($"✅ Wrote STRING: '{value}' to {tagName}");
                        break;

                    case "UDT":
                        Log("❌ UDT writing not supported in this example");
                        break;

                    default:
                        Log($"❌ Unsupported type: {type}");
                        break;
                }
            }
            catch (Exception ex)
            {
                Log($"❌ Write error: {ex.Message}");
            }
        }

        // Batch Read Event Handler
        private void BatchReadButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var tagListTextBox = (TextBox)Controls.Find("batchReadTagsTextBox", true)[0];
            var resultsListView = (ListView)Controls.Find("batchReadResultsListView", true)[0];
            var performanceLabel = (Label)Controls.Find("batchReadPerformanceLabel", true)[0];

            var tagNames = tagListTextBox.Text.Split('\n')
                .Select(line => line.Trim())
                .Where(line => !string.IsNullOrEmpty(line))
                .ToArray();

            if (tagNames.Length == 0)
            {
                Log("❌ Please enter at least one tag name");
                return;
            }

            try
            {
                Log($"🚀 Executing batch read for {tagNames.Length} tags...");
                var stopwatch = Stopwatch.StartNew();

                var results = _plcClient.ReadTagsBatch(tagNames);
                
                stopwatch.Stop();
                var totalTime = stopwatch.ElapsedMilliseconds;

                resultsListView.Items.Clear();
                int successCount = 0;

                foreach (var result in results)
                {
                    var item = new ListViewItem(result.Key);
                    
                    if (result.Value.Success)
                    {
                        item.SubItems.Add(result.Value.Value?.ToString() ?? "null");
                        item.SubItems.Add(result.Value.DataType);
                        item.SubItems.Add("✅ Success");
                        item.BackColor = Color.FromArgb(240, 253, 244);
                        successCount++;
                    }
                    else
                    {
                        item.SubItems.Add("Error");
                        item.SubItems.Add("N/A");
                        item.SubItems.Add($"❌ {result.Value.ErrorMessage}");
                        item.BackColor = Color.FromArgb(254, 242, 242);
                    }

                    resultsListView.Items.Add(item);
                }

                performanceLabel.Text = $"⏱️ Performance: {totalTime}ms total, {(double)totalTime / tagNames.Length:F1}ms avg/tag";
                Log($"✅ Batch read completed: {successCount}/{tagNames.Length} successful in {totalTime}ms");
                Log($"📈 Average time per tag: {(double)totalTime / tagNames.Length:F1}ms");
            }
            catch (Exception ex)
            {
                Log($"❌ Batch read error: {ex.Message}");
                performanceLabel.Text = "⏱️ Performance: Error occurred";
            }
        }

        // Batch Write Event Handler
        private void BatchWriteButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var tagValueTextBox = (TextBox)Controls.Find("batchWriteTagsTextBox", true)[0];
            var resultsListView = (ListView)Controls.Find("batchWriteResultsListView", true)[0];
            var performanceLabel = (Label)Controls.Find("batchWritePerformanceLabel", true)[0];

            var lines = tagValueTextBox.Text.Split('\n')
                .Select(line => line.Trim())
                .Where(line => !string.IsNullOrEmpty(line) && line.Contains('='))
                .ToArray();

            if (lines.Length == 0)
            {
                Log("❌ Please enter tag=value pairs (one per line)");
                return;
            }

            var tagValues = new Dictionary<string, object>();

            foreach (var line in lines)
            {
                var parts = line.Split('=', 2);
                if (parts.Length == 2)
                {
                    var tagName = parts[0].Trim();
                    var valueStr = parts[1].Trim();

                    // Try to parse the value as different types
                    object value;
                    if (bool.TryParse(valueStr, out bool boolVal))
                        value = boolVal;
                    else if (int.TryParse(valueStr, out int intVal))
                        value = intVal;
                    else if (float.TryParse(valueStr, out float floatVal))
                        value = floatVal;
                    else
                        value = valueStr; // String

                    tagValues[tagName] = value;
                }
            }

            try
            {
                Log($"✏️ Executing batch write for {tagValues.Count} tags...");
                var stopwatch = Stopwatch.StartNew();

                var results = _plcClient.WriteTagsBatch(tagValues);
                
                stopwatch.Stop();
                var totalTime = stopwatch.ElapsedMilliseconds;

                resultsListView.Items.Clear();
                int successCount = 0;

                foreach (var result in results)
                {
                    var item = new ListViewItem(result.Key);
                    var originalValue = tagValues.ContainsKey(result.Key) ? tagValues[result.Key] : "Unknown";
                    
                    item.SubItems.Add(originalValue.ToString());
                    item.SubItems.Add(originalValue.GetType().Name);
                    
                    if (result.Value.Success)
                    {
                        item.SubItems.Add("✅ Write successful");
                        item.BackColor = Color.FromArgb(240, 253, 244);
                        successCount++;
                    }
                    else
                    {
                        item.SubItems.Add($"❌ {result.Value.ErrorMessage}");
                        item.BackColor = Color.FromArgb(254, 242, 242);
                    }

                    resultsListView.Items.Add(item);
                }

                performanceLabel.Text = $"⏱️ Performance: {totalTime}ms total, {(double)totalTime / tagValues.Count:F1}ms avg/tag";
                Log($"✅ Batch write completed: {successCount}/{tagValues.Count} successful in {totalTime}ms");
            }
            catch (Exception ex)
            {
                Log($"❌ Batch write error: {ex.Message}");
                performanceLabel.Text = "⏱️ Performance: Error occurred";
            }
        }

        // Mixed Batch Event Handler
        private void MixedBatchButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var operationsTextBox = (TextBox)Controls.Find("mixedBatchOperationsTextBox", true)[0];
            var resultsListView = (ListView)Controls.Find("mixedBatchResultsListView", true)[0];
            var performanceLabel = (Label)Controls.Find("mixedBatchPerformanceLabel", true)[0];

            var lines = operationsTextBox.Text.Split('\n')
                .Select(line => line.Trim())
                .Where(line => !string.IsNullOrEmpty(line))
                .ToArray();

            if (lines.Length == 0)
            {
                Log("❌ Please enter operations (READ:TagName or WRITE:TagName=Value)");
                return;
            }

            var operations = new List<BatchOperation>();

            foreach (var line in lines)
            {
                if (line.StartsWith("READ:", StringComparison.OrdinalIgnoreCase))
                {
                    var tagName = line.Substring(5).Trim();
                    operations.Add(BatchOperation.Read(tagName));
                }
                else if (line.StartsWith("WRITE:", StringComparison.OrdinalIgnoreCase))
                {
                    var writeSpec = line.Substring(6).Trim();
                    var parts = writeSpec.Split('=', 2);
                    if (parts.Length == 2)
                    {
                        var tagName = parts[0].Trim();
                        var valueStr = parts[1].Trim();

                        // Parse value
                        object value;
                        if (bool.TryParse(valueStr, out bool boolVal))
                            value = boolVal;
                        else if (int.TryParse(valueStr, out int intVal))
                            value = intVal;
                        else if (float.TryParse(valueStr, out float floatVal))
                            value = floatVal;
                        else
                            value = valueStr;

                        operations.Add(BatchOperation.Write(tagName, value));
                    }
                }
            }

            if (operations.Count == 0)
            {
                Log("❌ No valid operations found");
                return;
            }

            try
            {
                Log($"🔄 Executing mixed batch with {operations.Count} operations...");
                var stopwatch = Stopwatch.StartNew();

                var results = _plcClient.ExecuteBatch(operations.ToArray());
                
                stopwatch.Stop();
                var totalTime = stopwatch.ElapsedMilliseconds;

                resultsListView.Items.Clear();
                int successCount = 0;

                for (int i = 0; i < results.Length; i++)
                {
                    var result = results[i];
                    var operation = operations[i];

                    var item = new ListViewItem(operation.IsWrite ? "WRITE" : "READ");
                    item.SubItems.Add(operation.TagName);
                    
                    if (operation.IsWrite)
                    {
                        item.SubItems.Add(operation.Value?.ToString() ?? "null");
                    }
                    else
                    {
                        item.SubItems.Add(result.Success ? result.Value?.ToString() ?? "null" : "Error");
                    }

                    item.SubItems.Add($"{result.ExecutionTimeMs * 1000:F0}"); // Convert to microseconds
                    
                    if (result.Success)
                    {
                        item.SubItems.Add("✅ Success");
                        item.BackColor = Color.FromArgb(240, 253, 244);
                        successCount++;
                    }
                    else
                    {
                        item.SubItems.Add($"❌ {result.ErrorMessage}");
                        item.BackColor = Color.FromArgb(254, 242, 242);
                    }

                    resultsListView.Items.Add(item);
                }

                performanceLabel.Text = $"⏱️ Performance: {totalTime}ms total, {(double)totalTime / operations.Count:F1}ms avg/op";
                Log($"✅ Mixed batch completed: {successCount}/{operations.Count} successful in {totalTime}ms");
            }
            catch (Exception ex)
            {
                Log($"❌ Mixed batch error: {ex.Message}");
                performanceLabel.Text = "⏱️ Performance: Error occurred";
            }
        }

        // Configuration Event Handlers
        private void DefaultConfigButton_Click(object? sender, EventArgs e)
        {
            if (_plcClient == null) return;

            try
            {
                _plcClient.ConfigureBatchOperations(BatchConfig.Default());
                UpdateCurrentConfigDisplay();
                Log("📊 Applied default batch configuration");
            }
            catch (Exception ex)
            {
                Log($"❌ Error applying default config: {ex.Message}");
            }
        }

        private void HighPerfConfigButton_Click(object? sender, EventArgs e)
        {
            if (_plcClient == null) return;

            try
            {
                _plcClient.ConfigureBatchOperations(BatchConfig.HighPerformance());
                UpdateCurrentConfigDisplay();
                Log("🚀 Applied high-performance batch configuration");
            }
            catch (Exception ex)
            {
                Log($"❌ Error applying high-performance config: {ex.Message}");
            }
        }

        private void ConservativeConfigButton_Click(object? sender, EventArgs e)
        {
            if (_plcClient == null) return;

            try
            {
                _plcClient.ConfigureBatchOperations(BatchConfig.Conservative());
                UpdateCurrentConfigDisplay();
                Log("🛡️ Applied conservative batch configuration");
            }
            catch (Exception ex)
            {
                Log($"❌ Error applying conservative config: {ex.Message}");
            }
        }

        private void ApplyCustomConfigButton_Click(object? sender, EventArgs e)
        {
            if (_plcClient == null) return;

            try
            {
                var maxOpsNumeric = (NumericUpDown)Controls.Find("maxOpsNumeric", true)[0];
                var maxPacketNumeric = (NumericUpDown)Controls.Find("maxPacketNumeric", true)[0];
                var timeoutNumeric = (NumericUpDown)Controls.Find("timeoutNumeric", true)[0];
                var continueOnErrorCheck = (CheckBox)Controls.Find("continueOnErrorCheck", true)[0];
                var optimizePackingCheck = (CheckBox)Controls.Find("optimizePackingCheck", true)[0];

                var customConfig = new BatchConfig
                {
                    MaxOperationsPerPacket = (int)maxOpsNumeric.Value,
                    MaxPacketSize = (int)maxPacketNumeric.Value,
                    PacketTimeoutMs = (long)timeoutNumeric.Value,
                    ContinueOnError = continueOnErrorCheck.Checked,
                    OptimizePacketPacking = optimizePackingCheck.Checked
                };

                _plcClient.ConfigureBatchOperations(customConfig);
                UpdateCurrentConfigDisplay();
                Log("🔧 Applied custom batch configuration");
            }
            catch (Exception ex)
            {
                Log($"❌ Error applying custom config: {ex.Message}");
            }
        }

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            base.OnFormClosing(e);

            if (_plcClient != null)
            {
                _plcClient.Dispose();
            }

            _connectionMonitorTimer?.Dispose();
        }
    }

    public class TagInfo
    {
        public required string Name { get; set; }
        public required object Value { get; set; }
        public required string Type { get; set; }
        public DateTime Updated { get; set; }
    }
} 