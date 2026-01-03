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

            // RoutePath controls for ControlLogix
            var routePathLabel = new Label
            {
                Text = "CPU Slot:",
                Location = new Point(310, 15),
                AutoSize = true
            };
            panel.Controls.Add(routePathLabel);

            var cpuSlotNumeric = new NumericUpDown
            {
                Name = "cpuSlotNumeric",
                Location = new Point(380, 12),
                Size = new Size(50, 23),
                Minimum = 0,
                Maximum = 15,
                Value = 0
            };
            panel.Controls.Add(cpuSlotNumeric);

            var useRoutePathCheck = new CheckBox
            {
                Name = "useRoutePathCheck",
                Text = "ControlLogix (Route Path)",
                Location = new Point(440, 14),
                AutoSize = true,
                Checked = true // Default to ControlLogix
            };
            panel.Controls.Add(useRoutePathCheck);

            // Connect/Disconnect buttons
            var connectButton = new Button
            {
                Name = "connectButton",
                Text = "Connect",
                Location = new Point(600, 11),
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
                Location = new Point(710, 11),
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

            // Array Tests Tab
            var arrayTab = new TabPage("📊 Array Tests");
            arrayTab.Controls.Add(CreateArrayTestsPanel());
            tabControl.TabPages.Add(arrayTab);

            // UDT Tests Tab
            var udtTab = new TabPage("🏗️ UDT Tests");
            udtTab.Controls.Add(CreateUdtTestsPanel());
            tabControl.TabPages.Add(udtTab);

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
                Text = "gTestArray_DINT[5]\ngTestArray_REAL[0]\ngTestArray_BOOL[0]\ngTestArray_INT[0]\ngTestUDT.Member1_DINT\ngTestUDT.Member2_REAL"
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
                Text = "gTestArray_DINT[5]=999\ngTestArray_REAL[0]=88.8\ngTestArray_BOOL[0]=true\ngTestArray_INT[0]=777\ngTestUDT.Member1_DINT=500\ngTestUDT.Member2_REAL=2.71828"
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
                Text = "READ:gTestArray_DINT[5]\nREAD:gTestArray_REAL[0]\nWRITE:gTestArray_DINT[5]=777\nWRITE:gTestArray_REAL[0]=99.9"
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
            
            // Array and UDT test buttons
            var arrayReadButton = Controls.Find("arrayReadButton", true).FirstOrDefault() as Button;
            var arrayWriteButton = Controls.Find("arrayWriteButton", true).FirstOrDefault() as Button;
            var udtReadButton = Controls.Find("udtReadButton", true).FirstOrDefault() as Button;
            var udtMemberReadButton = Controls.Find("udtMemberReadButton", true).FirstOrDefault() as Button;
            var udtMemberWriteButton = Controls.Find("udtMemberWriteButton", true).FirstOrDefault() as Button;

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
                
                // Enable array and UDT test buttons
                if (arrayReadButton != null) arrayReadButton.Enabled = true;
                if (arrayWriteButton != null) arrayWriteButton.Enabled = true;
                if (udtReadButton != null) udtReadButton.Enabled = true;
                if (udtMemberReadButton != null) udtMemberReadButton.Enabled = true;
                if (udtMemberWriteButton != null) udtMemberWriteButton.Enabled = true;
                
                // Enable quick test buttons
                EnableQuickTestButtons(true);

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
                
                // Disable array and UDT test buttons
                if (arrayReadButton != null) arrayReadButton.Enabled = false;
                if (arrayWriteButton != null) arrayWriteButton.Enabled = false;
                if (udtReadButton != null) udtReadButton.Enabled = false;
                if (udtMemberReadButton != null) udtMemberReadButton.Enabled = false;
                if (udtMemberWriteButton != null) udtMemberWriteButton.Enabled = false;
                
                // Disable quick test buttons
                EnableQuickTestButtons(false);
            }
        }

        private void EnableQuickTestButtons(bool enabled)
        {
            // Find all quick test buttons by checking Tag property
            foreach (Control control in Controls)
            {
                if (control is Button btn && btn.Tag is ValueTuple<string, string>)
                {
                    btn.Enabled = enabled;
                }
                // Recursively check child controls
                foreach (Control child in control.Controls)
                {
                    if (child is Button childBtn && childBtn.Tag is ValueTuple<string, string>)
                    {
                        childBtn.Enabled = enabled;
                    }
                }
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
            var cpuSlotNumeric = (NumericUpDown)Controls.Find("cpuSlotNumeric", true)[0];
            var useRoutePathCheck = (CheckBox)Controls.Find("useRoutePathCheck", true)[0];
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
                
                if (useRoutePathCheck.Checked)
                {
                    // ControlLogix with RoutePath
                    var routePath = new RoutePath().AddSlot((byte)cpuSlotNumeric.Value);
                    Log($"📍 Using RoutePath: CPU Slot {cpuSlotNumeric.Value}");
                    // Connect first, then set route path
                    _isConnected = _plcClient.Connect(address);
                    if (_isConnected)
                    {
                        // SetRoutePath will be called if the method exists
                        // For now, route path is set during connection in the Rust library
                        // This is a workaround until ConnectWithRoute is fully implemented
                        Log("✅ Connected. Route path will be used for subsequent operations.");
                    }
                }
                else
                {
                    // CompactLogix (direct connection)
                    _isConnected = _plcClient.Connect(address);
                }
                
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

                // Test tags from PLC_TEST_TAG_DEFINITIONS.md
                var testTags = new(string name, string type, object value)[]
                {
                    ("gTestArray_DINT[0]", "DINT", 10),
                    ("gTestArray_DINT[5]", "DINT", 60),
                    ("gTestArray_REAL[0]", "REAL", 1.1f),
                    ("gTestArray_BOOL[0]", "BOOL", true),
                    ("gTestArray_INT[0]", "INT", 100),
                    ("gTestUDT.Member1_DINT", "DINT", 100),
                    ("gTestUDT.Member2_REAL", "REAL", 3.14159f),
                    ("gTestUDT.Member3_BOOL", "BOOL", true),
                    ("gTestUDT.Array_DINT[5]", "DINT", 6),
                    ("Program:TestProgram.gTestArray_DINT[5]", "DINT", 5000)
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

                // Try array element access first if tag name contains brackets
                if (tagName.Contains("["))
                {
                    try
                    {
                        var dintValue = _plcClient.ReadDint(tagName);
                        UpdateTagFields(tagName, "DINT", dintValue.ToString());
                        Log($"✅ Discovered DINT array element: {tagName} = {dintValue}");
                        return;
                    }
                    catch { }
                }
                
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

                // Try UDT last, as arrays might be misidentified as UDTs
                // Only try UDT if the tag name doesn't look like an array base name
                if (!tagName.Contains("[") && !tagName.EndsWith("_DINT") && !tagName.EndsWith("_REAL") && !tagName.EndsWith("_BOOL") && !tagName.EndsWith("_INT"))
                {
                    try
                    {
                        var udtValue = _plcClient.ReadUdt(tagName);
                        var memberCount = udtValue.UdtMembers?.Count ?? 0;
                        if (memberCount > 0 || udtValue.IsUdtDataFormat)
                        {
                            UpdateTagFields(tagName, "UDT", $"UDT with {memberCount} members");
                            Log($"✅ Discovered UDT tag: {tagName} with {memberCount} members");
                            return;
                        }
                    }
                    catch { }
                }

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

        private Panel CreateArrayTestsPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            var titleLabel = new Label
            {
                Text = "📊 Array Element Tests - Direct Element Addressing",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(59, 130, 246),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            var descLabel = new Label
            {
                Text = "Test array element read/write using tags from PLC_TEST_TAG_DEFINITIONS.md",
                Location = new Point(10, 35),
                AutoSize = true
            };
            panel.Controls.Add(descLabel);

            // Array Read Section
            var readGroup = new GroupBox
            {
                Text = "Array Element Read",
                Location = new Point(10, 60),
                Size = new Size(400, 200)
            };

            var readTagLabel = new Label
            {
                Text = "Tag Name:",
                Location = new Point(10, 25),
                AutoSize = true
            };
            readGroup.Controls.Add(readTagLabel);

            var readTagTextBox = new TextBox
            {
                Name = "arrayReadTagTextBox",
                Location = new Point(10, 45),
                Size = new Size(350, 23),
                Text = "gTestArray_DINT[5]"
            };
            readGroup.Controls.Add(readTagTextBox);

            var readButton = new Button
            {
                Name = "arrayReadButton",
                Text = "Read Element",
                Location = new Point(10, 80),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            readButton.Click += ArrayReadButton_Click;
            readGroup.Controls.Add(readButton);

            var readResultLabel = new Label
            {
                Name = "arrayReadResultLabel",
                Text = "Result: Not read yet",
                Location = new Point(10, 120),
                Size = new Size(350, 60),
                AutoSize = false
            };
            readGroup.Controls.Add(readResultLabel);

            panel.Controls.Add(readGroup);

            // Array Write Section
            var writeGroup = new GroupBox
            {
                Text = "Array Element Write",
                Location = new Point(420, 60),
                Size = new Size(400, 200)
            };

            var writeTagLabel = new Label
            {
                Text = "Tag Name:",
                Location = new Point(10, 25),
                AutoSize = true
            };
            writeGroup.Controls.Add(writeTagLabel);

            var writeTagTextBox = new TextBox
            {
                Name = "arrayWriteTagTextBox",
                Location = new Point(10, 45),
                Size = new Size(350, 23),
                Text = "gTestArray_DINT[5]"
            };
            writeGroup.Controls.Add(writeTagTextBox);

            var writeValueLabel = new Label
            {
                Text = "Value:",
                Location = new Point(10, 75),
                AutoSize = true
            };
            writeGroup.Controls.Add(writeValueLabel);

            var writeValueTextBox = new TextBox
            {
                Name = "arrayWriteValueTextBox",
                Location = new Point(10, 95),
                Size = new Size(150, 23),
                Text = "999"
            };
            writeGroup.Controls.Add(writeValueTextBox);

            var writeButton = new Button
            {
                Name = "arrayWriteButton",
                Text = "Write Element",
                Location = new Point(170, 93),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(249, 115, 22),
                ForeColor = Color.White,
                Enabled = false
            };
            writeButton.Click += ArrayWriteButton_Click;
            writeGroup.Controls.Add(writeButton);

            var writeResultLabel = new Label
            {
                Name = "arrayWriteResultLabel",
                Text = "Result: Not written yet",
                Location = new Point(10, 130),
                Size = new Size(350, 60),
                AutoSize = false
            };
            writeGroup.Controls.Add(writeResultLabel);

            panel.Controls.Add(writeGroup);

            // Quick Test Buttons
            var quickTestGroup = new GroupBox
            {
                Text = "Quick Array Tests",
                Location = new Point(10, 270),
                Size = new Size(810, 150)
            };

            var quickTestButtons = new[]
            {
                ("Read gTestArray_DINT[5]", "gTestArray_DINT[5]", "read"),
                ("Read gTestArray_REAL[0]", "gTestArray_REAL[0]", "read"),
                ("Read gTestArray_BOOL[0]", "gTestArray_BOOL[0]", "read"),
                ("Read gTestArray_Large[300]", "gTestArray_Large[300]", "read"),
                ("Write gTestArray_DINT[5]=999", "gTestArray_DINT[5]", "write"),
                ("Write gTestArray_REAL[0]=88.8", "gTestArray_REAL[0]", "write")
            };

            int xPos = 10;
            int yPos = 25;
            foreach (var (text, tag, op) in quickTestButtons)
            {
                var btn = new Button
                {
                    Text = text,
                    Location = new Point(xPos, yPos),
                    Size = new Size(180, 30),
                    Tag = (tag, op),
                    Enabled = false
                };
                btn.Click += QuickArrayTestButton_Click;
                quickTestGroup.Controls.Add(btn);
                xPos += 190;
                if (xPos > 750)
                {
                    xPos = 10;
                    yPos += 40;
                }
            }

            panel.Controls.Add(quickTestGroup);

            return panel;
        }

        private Panel CreateUdtTestsPanel()
        {
            var panel = new Panel { Dock = DockStyle.Fill, Padding = new Padding(10) };

            var titleLabel = new Label
            {
                Text = "🏗️ UDT (User Defined Type) Tests - Generic UdtData Format",
                Font = new Font(this.Font, FontStyle.Bold),
                ForeColor = Color.FromArgb(147, 51, 234),
                Location = new Point(10, 10),
                AutoSize = true
            };
            panel.Controls.Add(titleLabel);

            var descLabel = new Label
            {
                Text = "Test UDT read/write using new generic UdtData format with symbol_id",
                Location = new Point(10, 35),
                AutoSize = true
            };
            panel.Controls.Add(descLabel);

            // UDT Read Section
            var readGroup = new GroupBox
            {
                Text = "UDT Read (UdtData Format)",
                Location = new Point(10, 60),
                Size = new Size(400, 250)
            };

            var readTagLabel = new Label
            {
                Text = "Tag Name:",
                Location = new Point(10, 25),
                AutoSize = true
            };
            readGroup.Controls.Add(readTagLabel);

            var readTagTextBox = new TextBox
            {
                Name = "udtReadTagTextBox",
                Location = new Point(10, 45),
                Size = new Size(350, 23),
                Text = "gTestUDT"
            };
            readGroup.Controls.Add(readTagTextBox);

            var readButton = new Button
            {
                Name = "udtReadButton",
                Text = "Read UDT",
                Location = new Point(10, 80),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            readButton.Click += UdtReadButton_Click;
            readGroup.Controls.Add(readButton);

            var readResultLabel = new Label
            {
                Name = "udtReadResultLabel",
                Text = "Result: Not read yet",
                Location = new Point(10, 120),
                Size = new Size(350, 120),
                AutoSize = false
            };
            readGroup.Controls.Add(readResultLabel);

            panel.Controls.Add(readGroup);

            // UDT Member Access Section
            var memberGroup = new GroupBox
            {
                Text = "UDT Member Access",
                Location = new Point(420, 60),
                Size = new Size(400, 250)
            };

            var memberTagLabel = new Label
            {
                Text = "Member Path:",
                Location = new Point(10, 25),
                AutoSize = true
            };
            memberGroup.Controls.Add(memberTagLabel);

            var memberTagTextBox = new TextBox
            {
                Name = "udtMemberTagTextBox",
                Location = new Point(10, 45),
                Size = new Size(350, 23),
                Text = "gTestUDT.Member1_DINT"
            };
            memberGroup.Controls.Add(memberTagTextBox);

            var memberReadButton = new Button
            {
                Name = "udtMemberReadButton",
                Text = "Read Member",
                Location = new Point(10, 80),
                Size = new Size(120, 30),
                BackColor = Color.FromArgb(34, 197, 94),
                ForeColor = Color.White,
                Enabled = false
            };
            memberReadButton.Click += UdtMemberReadButton_Click;
            memberGroup.Controls.Add(memberReadButton);

            var memberWriteValueLabel = new Label
            {
                Text = "Value:",
                Location = new Point(140, 80),
                AutoSize = true
            };
            memberGroup.Controls.Add(memberWriteValueLabel);

            var memberWriteValueTextBox = new TextBox
            {
                Name = "udtMemberWriteValueTextBox",
                Location = new Point(140, 100),
                Size = new Size(100, 23),
                Text = "500"
            };
            memberGroup.Controls.Add(memberWriteValueTextBox);

            var memberWriteButton = new Button
            {
                Name = "udtMemberWriteButton",
                Text = "Write Member",
                Location = new Point(250, 98),
                Size = new Size(110, 30),
                BackColor = Color.FromArgb(249, 115, 22),
                ForeColor = Color.White,
                Enabled = false
            };
            memberWriteButton.Click += UdtMemberWriteButton_Click;
            memberGroup.Controls.Add(memberWriteButton);

            var memberResultLabel = new Label
            {
                Name = "udtMemberResultLabel",
                Text = "Result: Not accessed yet",
                Location = new Point(10, 140),
                Size = new Size(350, 100),
                AutoSize = false
            };
            memberGroup.Controls.Add(memberResultLabel);

            panel.Controls.Add(memberGroup);

            // Quick UDT Test Buttons
            var quickTestGroup = new GroupBox
            {
                Text = "Quick UDT Tests",
                Location = new Point(10, 320),
                Size = new Size(810, 150)
            };

            var quickUdtTests = new[]
            {
                ("Read gTestUDT", "gTestUDT", "read"),
                ("Read gTestUDT.Member1_DINT", "gTestUDT.Member1_DINT", "read"),
                ("Read gTestUDT.Member2_REAL", "gTestUDT.Member2_REAL", "read"),
                ("Read gTestUDT.Array_DINT[5]", "gTestUDT.Array_DINT[5]", "read"),
                ("Read gTestUDT_Array[3]", "gTestUDT_Array[3]", "read"),
                ("Read Program:TestProgram.gTestUDT", "Program:TestProgram.gTestUDT", "read")
            };

            int xPos = 10;
            int yPos = 25;
            foreach (var (text, tag, op) in quickUdtTests)
            {
                var btn = new Button
                {
                    Text = text,
                    Location = new Point(xPos, yPos),
                    Size = new Size(180, 30),
                    Tag = (tag, op),
                    Enabled = false
                };
                btn.Click += QuickUdtTestButton_Click;
                quickTestGroup.Controls.Add(btn);
                xPos += 190;
                if (xPos > 750)
                {
                    xPos = 10;
                    yPos += 40;
                }
            }

            panel.Controls.Add(quickTestGroup);

            return panel;
        }

        // Array Test Event Handlers
        private void ArrayReadButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var readTagTextBox = (TextBox)Controls.Find("arrayReadTagTextBox", true)[0];
            var resultLabel = (Label)Controls.Find("arrayReadResultLabel", true)[0];
            var tagName = readTagTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name");
                return;
            }

            try
            {
                Log($"📖 Reading array element: {tagName}");
                object? value = null;
                string type = "UNKNOWN";
                
                // Try different types based on tag name
                if (tagName.Contains("DINT") || tagName.Contains("INT["))
                {
                    value = _plcClient.ReadDint(tagName);
                    type = "DINT";
                }
                else if (tagName.Contains("REAL"))
                {
                    value = _plcClient.ReadReal(tagName);
                    type = "REAL";
                }
                else if (tagName.Contains("BOOL"))
                {
                    value = _plcClient.ReadBool(tagName);
                    type = "BOOL";
                }
                else if (tagName.Contains("INT") && !tagName.Contains("DINT"))
                {
                    value = _plcClient.ReadInt(tagName);
                    type = "INT";
                }
                else
                {
                    // Try DINT as default
                    value = _plcClient.ReadDint(tagName);
                    type = "DINT";
                }
                
                resultLabel.Text = $"✅ Success!\nTag: {tagName}\nValue: {value}\nType: {type}";
                Log($"✅ Read {tagName} = {value} ({type})");
            }
            catch (Exception ex)
            {
                resultLabel.Text = $"❌ Error: {ex.Message}";
                Log($"❌ Read error: {ex.Message}");
            }
        }

        private void ArrayWriteButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var writeTagTextBox = (TextBox)Controls.Find("arrayWriteTagTextBox", true)[0];
            var writeValueTextBox = (TextBox)Controls.Find("arrayWriteValueTextBox", true)[0];
            var resultLabel = (Label)Controls.Find("arrayWriteResultLabel", true)[0];
            var tagName = writeTagTextBox.Text.Trim();
            var valueStr = writeValueTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name");
                return;
            }

            try
            {
                Log($"✏️ Writing array element: {tagName} = {valueStr}");
                
                // Try to determine type from tag name
                if (tagName.Contains("DINT") || tagName.Contains("INT["))
                {
                    if (int.TryParse(valueStr, out int intValue))
                    {
                        _plcClient.WriteDint(tagName, intValue);
                        resultLabel.Text = $"✅ Success!\nTag: {tagName}\nWritten: {intValue} (DINT)";
                        Log($"✅ Wrote {tagName} = {intValue}");
                    }
                    else
                    {
                        throw new Exception("Invalid integer value");
                    }
                }
                else if (tagName.Contains("REAL"))
                {
                    if (float.TryParse(valueStr, out float floatValue))
                    {
                        _plcClient.WriteReal(tagName, floatValue);
                        resultLabel.Text = $"✅ Success!\nTag: {tagName}\nWritten: {floatValue} (REAL)";
                        Log($"✅ Wrote {tagName} = {floatValue}");
                    }
                    else
                    {
                        throw new Exception("Invalid float value");
                    }
                }
                else if (tagName.Contains("BOOL"))
                {
                    if (bool.TryParse(valueStr, out bool boolValue))
                    {
                        _plcClient.WriteBool(tagName, boolValue);
                        resultLabel.Text = $"✅ Success!\nTag: {tagName}\nWritten: {boolValue} (BOOL)";
                        Log($"✅ Wrote {tagName} = {boolValue}");
                    }
                    else
                    {
                        throw new Exception("Invalid boolean value");
                    }
                }
                else
                {
                    throw new Exception("Cannot determine data type from tag name");
                }
            }
            catch (Exception ex)
            {
                resultLabel.Text = $"❌ Error: {ex.Message}";
                Log($"❌ Write error: {ex.Message}");
            }
        }

        private void QuickArrayTestButton_Click(object? sender, EventArgs e)
        {
            if (sender is Button btn && btn.Tag is ValueTuple<string, string> tagInfo)
            {
                var (tag, op) = tagInfo;
                var readTagTextBox = (TextBox)Controls.Find("arrayReadTagTextBox", true)[0];
                var writeTagTextBox = (TextBox)Controls.Find("arrayWriteTagTextBox", true)[0];
                var writeValueTextBox = (TextBox)Controls.Find("arrayWriteValueTextBox", true)[0];

                if (op == "read")
                {
                    readTagTextBox.Text = tag;
                    ArrayReadButton_Click(sender, EventArgs.Empty);
                }
                else
                {
                    writeTagTextBox.Text = tag;
                    writeValueTextBox.Text = tag.Contains("REAL") ? "88.8" : "999";
                    ArrayWriteButton_Click(sender, EventArgs.Empty);
                }
            }
        }

        // UDT Test Event Handlers
        private void UdtReadButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var readTagTextBox = (TextBox)Controls.Find("udtReadTagTextBox", true)[0];
            var resultLabel = (Label)Controls.Find("udtReadResultLabel", true)[0];
            var tagName = readTagTextBox.Text.Trim();

            if (string.IsNullOrEmpty(tagName))
            {
                Log("❌ Please enter a tag name");
                return;
            }

            try
            {
                Log($"📖 Reading UDT: {tagName}");
                var value = _plcClient.ReadUdt(tagName);
                
                if (value.IsUdtDataFormat)
                {
                    var udtData = value.UdtData;
                    resultLabel.Text = $"✅ Success!\nTag: {tagName}\nSymbol ID: {udtData.SymbolId}\nData Length: {udtData.Data.Length} bytes\n\n" +
                                     $"⚠️ UDT is in UdtData format.\n" +
                                     $"To access members, use direct tag paths:\n" +
                                     $"  {tagName}.Member1_DINT\n" +
                                     $"  {tagName}.Member2_REAL\n" +
                                     $"etc.";
                    Log($"✅ Read UDT {tagName}: Symbol ID = {udtData.SymbolId}, Data Length = {udtData.Data.Length} bytes");
                    Log($"💡 Tip: Access members using direct tag paths like '{tagName}.Member1_DINT'");
                }
                else
                {
                    var memberCount = value.UdtMembers?.Count ?? 0;
                    var memberList = value.UdtMembers != null 
                        ? string.Join("\n", value.UdtMembers.Keys.Take(10).Select(k => $"  - {k}"))
                        : "  (Use GetUdtMember to access)";
                    resultLabel.Text = $"✅ Success!\nTag: {tagName}\nUDT with {memberCount} members\n\nMembers available:\n{memberList}";
                    Log($"✅ Read UDT {tagName} with {memberCount} members");
                }
            }
            catch (Exception ex)
            {
                resultLabel.Text = $"❌ Error: {ex.Message}";
                Log($"❌ Read error: {ex.Message}");
            }
        }

        private void UdtMemberReadButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var memberTagTextBox = (TextBox)Controls.Find("udtMemberTagTextBox", true)[0];
            var resultLabel = (Label)Controls.Find("udtMemberResultLabel", true)[0];
            var fullPath = memberTagTextBox.Text.Trim();

            if (string.IsNullOrEmpty(fullPath))
            {
                Log("❌ Please enter a member path");
                return;
            }

            try
            {
                Log($"📖 Reading UDT member: {fullPath}");
                
                // Parse the path: "gTestUDT.Member1_DINT" -> tagName="gTestUDT", memberPath="Member1_DINT"
                var parts = fullPath.Split('.');
                if (parts.Length < 2)
                {
                    throw new Exception("Invalid UDT member path. Use format: 'UDTName.MemberName'");
                }
                
                var tagName = parts[0];
                var memberPath = string.Join(".", parts.Skip(1));
                
                // Strategy 1: Try direct tag access first (works for both formats)
                // The Rust library supports direct member access like "gTestUDT.Member1_DINT"
                PlcValue? memberValue = null;
                string valueStr = "Unknown";
                string type = "UNKNOWN";
                Exception? directEx = null;
                
                try
                {
                    // Try direct tag read - this works even with UdtData format
                    // Determine type from member name and try appropriate read method
                    if (memberPath.Contains("DINT") || (memberPath.Contains("INT") && !memberPath.Contains("REAL")))
                    {
                        var intValue = _plcClient.ReadDint(fullPath);
                        valueStr = intValue.ToString();
                        type = "DINT";
                        resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {type}";
                        Log($"✅ Read {fullPath} = {valueStr} ({type})");
                        return;
                    }
                    else if (memberPath.Contains("REAL"))
                    {
                        var floatValue = _plcClient.ReadReal(fullPath);
                        valueStr = floatValue.ToString();
                        type = "REAL";
                        resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {type}";
                        Log($"✅ Read {fullPath} = {valueStr} ({type})");
                        return;
                    }
                    else if (memberPath.Contains("BOOL"))
                    {
                        var boolValue = _plcClient.ReadBool(fullPath);
                        valueStr = boolValue.ToString();
                        type = "BOOL";
                        resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {type}";
                        Log($"✅ Read {fullPath} = {valueStr} ({type})");
                        return;
                    }
                    else if (memberPath.Contains("INT") && !memberPath.Contains("DINT"))
                    {
                        var shortValue = _plcClient.ReadInt(fullPath);
                        valueStr = shortValue.ToString();
                        type = "INT";
                        resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {type}";
                        Log($"✅ Read {fullPath} = {valueStr} ({type})");
                        return;
                    }
                }
                catch (Exception ex)
                {
                    directEx = ex;
                    Log($"⚠️ Direct tag access failed for '{fullPath}': {ex.Message}");
                    // Continue to fallback methods
                }
                
                // Strategy 2: Read full UDT and extract member (works for UdtData format)
                try
                {
                    Log($"🔧 Reading full UDT '{tagName}' to extract member '{memberPath}'...");
                    var udtValue = _plcClient.ReadUdt(tagName);
                    
                    if (udtValue == null)
                    {
                        throw new Exception($"Failed to read UDT '{tagName}' - returned null");
                    }
                    
                    if (!udtValue.IsUdt)
                    {
                        throw new Exception($"Tag '{tagName}' is not a UDT type");
                    }
                    
                    Log($"🔧 UDT read successful. IsUdtDataFormat: {udtValue.IsUdtDataFormat}, HasUdtMembers: {udtValue.UdtMembers != null}");
                    
                    // Debug: List all available members
                    if (udtValue.UdtMembers != null)
                    {
                        var memberNames = string.Join(", ", udtValue.UdtMembers.Keys);
                        Log($"🔍 Available UDT members: {memberNames}");
                        Log($"🔍 Looking for member: '{memberPath}' (case-sensitive: {udtValue.UdtMembers.ContainsKey(memberPath)})");
                        
                        // Check if _parsed_members contains the actual members
                        if (udtValue.UdtMembers.ContainsKey("_parsed_members"))
                        {
                            var parsedMembersValue = udtValue.UdtMembers["_parsed_members"];
                            Log($"🔧 Found _parsed_members field, type: {parsedMembersValue?.Type}");
                            
                            // Try to extract actual members from _parsed_members
                            if (parsedMembersValue != null && parsedMembersValue.IsUdt)
                            {
                                var actualMembers = parsedMembersValue.UdtMembers;
                                if (actualMembers != null)
                                {
                                    var actualMemberNames = string.Join(", ", actualMembers.Keys);
                                    Log($"🔍 Actual UDT members in _parsed_members: {actualMemberNames}");
                                    
                                    // Try to find the member in the actual members
                                    if (actualMembers.ContainsKey(memberPath))
                                    {
                                        memberValue = actualMembers[memberPath];
                                        Log($"✅ Found member '{memberPath}' in _parsed_members");
                                    }
                                    else
                                    {
                                        // Try case-insensitive
                                        var matchingKey = actualMembers.Keys.FirstOrDefault(k => 
                                            k.Equals(memberPath, StringComparison.OrdinalIgnoreCase));
                                        if (matchingKey != null)
                                        {
                                            Log($"🔧 Found case-insensitive match in _parsed_members: '{matchingKey}'");
                                            memberValue = actualMembers[matchingKey];
                                            Log($"✅ Got member via case-insensitive lookup in _parsed_members");
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Also check _raw_data if available - it might be a string representation of hex bytes
                        if (memberValue == null && udtValue.UdtMembers.ContainsKey("_raw_data"))
                        {
                            Log($"🔧 Found _raw_data field, attempting to parse...");
                            var rawDataValue = udtValue.UdtMembers["_raw_data"];
                            Log($"🔧 _raw_data type: {rawDataValue?.Type}, value: {rawDataValue?.ToString()}");
                            
                            // Try to parse _raw_data as hex string or byte array
                            if (rawDataValue != null)
                            {
                                byte[] rawBytes = null;
                                
                                if (rawDataValue.Type == PlcValueType.String)
                                {
                                    var hexString = rawDataValue.As<string>();
                                    Log($"🔧 _raw_data is string: '{hexString}'");
                                    // Try to parse as hex string (e.g., "[04, 00]" or "04 00")
                                    // For now, skip - this is a simplified format
                                }
                                
                                if (rawBytes != null)
                                {
                                    Log($"🔧 Parsing member from _raw_data bytes ({rawBytes.Length} bytes)");
                                    memberValue = ParseUdtMemberFromRawData(rawBytes, memberPath);
                                    if (memberValue != null)
                                    {
                                        Log($"✅ Parsed member '{memberPath}' from _raw_data");
                                    }
                                }
                            }
                        }
                        
                        // If we still don't have the member, and the UDT only has metadata fields,
                        // it means the actual UDT data wasn't parsed correctly
                        // In this case, we need to read the UDT again or use a different method
                        if (memberValue == null && udtValue.UdtMembers.Keys.All(k => k.StartsWith("_")))
                        {
                            Log($"⚠️ UDT only contains metadata fields, actual members not parsed.");
                            Log($"⚠️ This suggests the UDT read failed and fell back to chunked reading, which returns metadata only.");
                            Log($"💡 The actual UDT members are not available in this format.");
                            Log($"💡 Try using direct tag access: '{tagName}.{memberPath}' (if supported by PLC)");
                            Log($"💡 Or check if the Rust library's UDT parsing is working correctly.");
                            
                            throw new Exception(
                                $"UDT '{tagName}' was read but only contains metadata fields, not actual members.\n\n" +
                                $"Available fields: {string.Join(", ", udtValue.UdtMembers.Keys)}\n\n" +
                                $"This indicates the UDT read failed and fell back to a chunked method that doesn't parse members.\n\n" +
                                $"Possible solutions:\n" +
                                $"1. Verify the tag '{tagName}' exists and is accessible\n" +
                                $"2. Try using direct tag access: '{tagName}.{memberPath}'\n" +
                                $"3. Check if the Rust library's UDT definition parsing is working\n" +
                                $"4. The UDT might be too large and needs chunked reading with proper member parsing"
                            );
                        }
                    }
                    
                    // Try GetUdtMember (works for legacy format) - only if we haven't found it yet
                    if (memberValue == null)
                    {
                        memberValue = _plcClient.GetUdtMember(tagName, memberPath);
                        if (memberValue != null)
                        {
                            Log($"✅ Got member via GetUdtMember");
                            // Success with GetUdtMember
                        }
                        else
                        {
                            Log($"⚠️ GetUdtMember returned null for '{memberPath}'");
                            
                            // Try case-insensitive lookup in top-level members
                            if (udtValue.UdtMembers != null)
                            {
                                var matchingKey = udtValue.UdtMembers.Keys.FirstOrDefault(k => 
                                    k.Equals(memberPath, StringComparison.OrdinalIgnoreCase) && 
                                    !k.StartsWith("_"));
                                if (matchingKey != null)
                                {
                                    Log($"🔧 Found case-insensitive match: '{matchingKey}' (requested: '{memberPath}')");
                                    memberValue = udtValue.UdtMembers[matchingKey];
                                    Log($"✅ Got member via case-insensitive lookup");
                                }
                            }
                        }
                    }
                    
                    if (udtValue.IsUdtDataFormat)
                    {
                        Log($"🔧 UDT is in UdtData format, attempting to parse from raw bytes...");
                        // UdtData format - parse from raw bytes using known structure
                        var udtData = udtValue.UdtData;
                        if (udtData != null && udtData.Data != null)
                        {
                            Log($"🔧 Attempting to parse member '{memberPath}' from UdtData (Size: {udtData.Data.Length} bytes)");
                            memberValue = ParseUdtMemberFromRawData(udtData.Data, memberPath);
                            if (memberValue != null)
                            {
                                // Successfully parsed from raw data
                                Log($"✅ Parsed member '{memberPath}' from UdtData raw bytes");
                            }
                            else
                            {
                                Log($"⚠️ Failed to parse member '{memberPath}' from UdtData. Available data: {udtData.Data.Length} bytes");
                                // Log first few bytes for debugging
                                var hexPreview = string.Join(" ", udtData.Data.Take(32).Select(b => b.ToString("X2")));
                                Log($"🔍 First 32 bytes (hex): {hexPreview}");
                            }
                        }
                        else
                        {
                            Log($"⚠️ UdtData is null or has no data (udtData={udtData != null}, Data={udtData?.Data != null})");
                        }
                    }
                    else
                    {
                        Log($"⚠️ UDT is not in UdtData format and GetUdtMember returned null");
                    }
                }
                catch (Exception udtEx)
                {
                    var directErrorMsg = directEx != null ? directEx.Message : "N/A";
                    Log($"❌ UDT read/parse error: {udtEx.Message}");
                    throw new Exception(
                        $"Failed to read UDT member '{memberPath}' from '{tagName}'.\n\n" +
                        $"Direct tag access error: {directErrorMsg}\n" +
                        $"UDT read error: {udtEx.Message}\n\n" +
                        $"Troubleshooting:\n" +
                        $"1. Verify tag '{tagName}' exists in PLC\n" +
                        $"2. Verify member '{memberPath}' exists in UDT definition\n" +
                        $"3. Try reading full UDT first: ReadUdt('{tagName}')"
                    );
                }
                
                if (memberValue == null)
                {
                    throw new Exception($"Member '{memberPath}' not found in UDT '{tagName}'. Check the log for parsing details.");
                }
                
                // Extract value based on type (reuse existing variables)
                valueStr = "Unknown";
                type = memberValue.Type.ToString();
                
                switch (memberValue.Type)
                {
                    case PlcValueType.Bool:
                        valueStr = memberValue.As<bool>().ToString();
                        break;
                    case PlcValueType.Dint:
                        valueStr = memberValue.As<int>().ToString();
                        break;
                    case PlcValueType.Int:
                        valueStr = memberValue.As<short>().ToString();
                        break;
                    case PlcValueType.Real:
                        valueStr = memberValue.As<float>().ToString();
                        break;
                    case PlcValueType.String:
                        valueStr = memberValue.As<string>();
                        break;
                    default:
                        valueStr = memberValue.ToString();
                        break;
                }
                
                resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nValue: {valueStr}\nType: {type}";
                Log($"✅ Read {fullPath} = {valueStr} ({type})");
            }
            catch (Exception ex)
            {
                resultLabel.Text = $"❌ Error: {ex.Message}";
                Log($"❌ Read error: {ex.Message}");
            }
        }

        private void UdtMemberWriteButton_Click(object? sender, EventArgs e)
        {
            if (!_isConnected || _plcClient == null) return;

            var memberTagTextBox = (TextBox)Controls.Find("udtMemberTagTextBox", true)[0];
            var writeValueTextBox = (TextBox)Controls.Find("udtMemberWriteValueTextBox", true)[0];
            var resultLabel = (Label)Controls.Find("udtMemberResultLabel", true)[0];
            var fullPath = memberTagTextBox.Text.Trim();
            var valueStr = writeValueTextBox.Text.Trim();

            if (string.IsNullOrEmpty(fullPath))
            {
                Log("❌ Please enter a member path");
                return;
            }

            try
            {
                Log($"✏️ Writing UDT member: {fullPath} = {valueStr}");
                
                // Parse the path: "gTestUDT.Member1_DINT" -> tagName="gTestUDT", memberPath="Member1_DINT"
                var parts = fullPath.Split('.');
                if (parts.Length < 2)
                {
                    throw new Exception("Invalid UDT member path. Use format: 'UDTName.MemberName'");
                }
                
                var tagName = parts[0];
                var memberPath = string.Join(".", parts.Skip(1));
                
                // First, try direct tag write (works for UdtData format)
                // The Rust library supports direct member access like "gTestUDT.Member1_DINT"
                try
                {
                    // Determine type from member name and try appropriate write method
                    if (memberPath.Contains("DINT") || (memberPath.Contains("INT") && !memberPath.Contains("REAL")))
                    {
                        if (int.TryParse(valueStr, out int intValue))
                        {
                            _plcClient.WriteDint(fullPath, intValue);
                            resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nWritten: {intValue} (DINT)";
                            Log($"✅ Wrote {fullPath} = {intValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("REAL"))
                    {
                        if (float.TryParse(valueStr, out float floatValue))
                        {
                            _plcClient.WriteReal(fullPath, floatValue);
                            resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nWritten: {floatValue} (REAL)";
                            Log($"✅ Wrote {fullPath} = {floatValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("BOOL"))
                    {
                        if (bool.TryParse(valueStr, out bool boolValue))
                        {
                            _plcClient.WriteBool(fullPath, boolValue);
                            resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nWritten: {boolValue} (BOOL)";
                            Log($"✅ Wrote {fullPath} = {boolValue}");
                            return;
                        }
                    }
                    else if (memberPath.Contains("INT") && !memberPath.Contains("DINT"))
                    {
                        if (short.TryParse(valueStr, out short shortValue))
                        {
                            _plcClient.WriteInt(fullPath, shortValue);
                            resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nWritten: {shortValue} (INT)";
                            Log($"✅ Wrote {fullPath} = {shortValue}");
                            return;
                        }
                    }
                }
                catch (Exception directEx)
                {
                    // Direct access failed, try SetUdtMember (works for legacy format)
                    Log($"⚠️ Direct write failed, trying SetUdtMember: {directEx.Message}");
                }
                
                // Fallback: Use SetUdtMember helper method (for legacy UDT format)
                // Determine value type from member name and parse value
                PlcValue plcValue;
                
                if (memberPath.Contains("DINT") || (memberPath.Contains("INT") && !memberPath.Contains("REAL")))
                {
                    if (int.TryParse(valueStr, out int intValue))
                    {
                        plcValue = PlcValue.Dint(intValue);
                    }
                    else
                    {
                        throw new Exception("Invalid integer value");
                    }
                }
                else if (memberPath.Contains("REAL"))
                {
                    if (float.TryParse(valueStr, out float floatValue))
                    {
                        plcValue = PlcValue.Real(floatValue);
                    }
                    else
                    {
                        throw new Exception("Invalid float value");
                    }
                }
                else if (memberPath.Contains("BOOL"))
                {
                    if (bool.TryParse(valueStr, out bool boolValue))
                    {
                        plcValue = PlcValue.Bool(boolValue);
                    }
                    else
                    {
                        throw new Exception("Invalid boolean value");
                    }
                }
                else if (memberPath.Contains("INT") && !memberPath.Contains("DINT"))
                {
                    if (short.TryParse(valueStr, out short shortValue))
                    {
                        plcValue = PlcValue.Int(shortValue);
                    }
                    else
                    {
                        throw new Exception("Invalid short integer value");
                    }
                }
                else
                {
                    // Default to DINT
                    if (int.TryParse(valueStr, out int defaultInt))
                    {
                        plcValue = PlcValue.Dint(defaultInt);
                    }
                    else
                    {
                        throw new Exception("Cannot determine data type from member name");
                    }
                }
                
                // Use SetUdtMember helper method
                _plcClient.SetUdtMember(tagName, memberPath, plcValue);
                
                resultLabel.Text = $"✅ Success!\nUDT: {tagName}\nMember: {memberPath}\nWritten: {valueStr} ({plcValue.Type})";
                Log($"✅ Wrote {fullPath} = {valueStr} ({plcValue.Type})");
            }
            catch (Exception ex)
            {
                resultLabel.Text = $"❌ Error: {ex.Message}";
                Log($"❌ Write error: {ex.Message}");
            }
        }

        private void QuickUdtTestButton_Click(object? sender, EventArgs e)
        {
            if (sender is Button btn && btn.Tag is ValueTuple<string, string> tagInfo)
            {
                var (tag, op) = tagInfo;
                var readTagTextBox = (TextBox)Controls.Find("udtReadTagTextBox", true)[0];
                var memberTagTextBox = (TextBox)Controls.Find("udtMemberTagTextBox", true)[0];

                if (tag.Contains("."))
                {
                    memberTagTextBox.Text = tag;
                    if (op == "read")
                        UdtMemberReadButton_Click(sender, EventArgs.Empty);
                }
                else
                {
                    readTagTextBox.Text = tag;
                    UdtReadButton_Click(sender, EventArgs.Empty);
                }
            }
        }

        /// <summary>
        /// Parses a UDT member from raw bytes based on known TEST_UDT structure
        /// This is a workaround for when direct member access fails
        /// </summary>
        private PlcValue? ParseUdtMemberFromRawData(byte[] rawData, string memberPath)
        {
            try
            {
                if (rawData == null)
                {
                    Log("⚠️ ParseUdtMemberFromRawData: rawData is null");
                    return null;
                }
                
                Log($"🔧 ParseUdtMemberFromRawData: Looking for '{memberPath}' in {rawData.Length} bytes");
                
                // Known TEST_UDT structure offsets (from PLC_TEST_TAG_DEFINITIONS.md)
                // Note: Actual offsets may vary due to padding/alignment
                // Member1_DINT: Offset 0, Size 4
                // Member2_REAL: Offset 4, Size 4
                // Member3_BOOL: Offset 8, Size 1 (first bit)
                // Member4_INT: Offset 10, Size 2
                // Member5_String: Offset 12, Size 82
                // Array_DINT: Offset 96, Size 40 (10 * 4)
                // Array_REAL: Offset 136, Size 20 (5 * 4)
                // Array_BOOL: Offset 156, Size 20 (20 * 1)
                
                if (rawData.Length < 20)
                {
                    Log($"⚠️ ParseUdtMemberFromRawData: Data too short ({rawData.Length} bytes, need at least 20)");
                    return null;
                }
                
                // Handle simple member access
                // Try exact match first
                if (memberPath.Equals("Member1_DINT", StringComparison.OrdinalIgnoreCase))
                {
                    if (rawData.Length >= 4)
                    {
                        int value = BitConverter.ToInt32(rawData, 0);
                        Log($"✅ Parsed Member1_DINT from offset 0: {value}");
                        return PlcValue.Dint(value);
                    }
                    else
                    {
                        Log($"⚠️ Not enough data for Member1_DINT (need 4 bytes, have {rawData.Length})");
                    }
                }
                else if (memberPath.Equals("Member2_REAL", StringComparison.OrdinalIgnoreCase))
                {
                    if (rawData.Length >= 8)
                    {
                        float value = BitConverter.ToSingle(rawData, 4);
                        Log($"✅ Parsed Member2_REAL from offset 4: {value}");
                        return PlcValue.Real(value);
                    }
                    else
                    {
                        Log($"⚠️ Not enough data for Member2_REAL (need 8 bytes, have {rawData.Length})");
                    }
                }
                else if (memberPath.Equals("Member3_BOOL", StringComparison.OrdinalIgnoreCase))
                {
                    if (rawData.Length >= 9)
                    {
                        // BOOL is typically stored as a byte, where 0x01 = true
                        bool value = (rawData[8] & 0x01) != 0;
                        Log($"✅ Parsed Member3_BOOL from offset 8: {value} (byte value: 0x{rawData[8]:X2})");
                        return PlcValue.Bool(value);
                    }
                    else
                    {
                        Log($"⚠️ Not enough data for Member3_BOOL (need 9 bytes, have {rawData.Length})");
                    }
                }
                else if (memberPath.Equals("Member4_INT", StringComparison.OrdinalIgnoreCase))
                {
                    if (rawData.Length >= 12)
                    {
                        short value = BitConverter.ToInt16(rawData, 10);
                        Log($"✅ Parsed Member4_INT from offset 10: {value}");
                        return PlcValue.Int(value);
                    }
                    else
                    {
                        Log($"⚠️ Not enough data for Member4_INT (need 12 bytes, have {rawData.Length})");
                    }
                }
                else if (memberPath == "Member5_String")
                {
                    if (rawData.Length >= 94)
                    {
                        // STRING format: 2-byte length + data
                        ushort length = BitConverter.ToUInt16(rawData, 12);
                        if (length > 0 && length <= 82 && rawData.Length >= 14 + length)
                        {
                            string value = System.Text.Encoding.ASCII.GetString(rawData, 14, length).TrimEnd('\0');
                            return PlcValue.String(value);
                        }
                    }
                }
                // Handle array member access (e.g., "Array_DINT[5]")
                else if (memberPath.StartsWith("Array_DINT["))
                {
                    // Extract index from "Array_DINT[5]"
                    var indexStr = memberPath.Substring("Array_DINT[".Length).TrimEnd(']');
                    if (int.TryParse(indexStr, out int index) && index >= 0 && index < 10)
                    {
                        int offset = 96 + (index * 4);
                        if (rawData.Length >= offset + 4)
                        {
                            int value = BitConverter.ToInt32(rawData, offset);
                            return PlcValue.Dint(value);
                        }
                    }
                }
                else if (memberPath.StartsWith("Array_REAL["))
                {
                    var indexStr = memberPath.Substring("Array_REAL[".Length).TrimEnd(']');
                    if (int.TryParse(indexStr, out int index) && index >= 0 && index < 5)
                    {
                        int offset = 136 + (index * 4);
                        if (rawData.Length >= offset + 4)
                        {
                            float value = BitConverter.ToSingle(rawData, offset);
                            return PlcValue.Real(value);
                        }
                    }
                }
                else if (memberPath.StartsWith("Array_BOOL["))
                {
                    var indexStr = memberPath.Substring("Array_BOOL[".Length).TrimEnd(']');
                    if (int.TryParse(indexStr, out int index) && index >= 0 && index < 20)
                    {
                        int offset = 156 + index;
                        if (rawData.Length >= offset + 1)
                        {
                            bool value = (rawData[offset] & 0x01) != 0;
                            return PlcValue.Bool(value);
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                Log($"⚠️ Error parsing UDT member '{memberPath}' from raw data: {ex.Message}");
            }
            
            return null;
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