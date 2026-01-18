using System.Drawing;
using System.Windows.Forms;

namespace WinFormsExample
{
    /// <summary>
    /// Industrial minimalistic UI theme for EtherNet/IP examples
    /// </summary>
    public static class IndustrialTheme
    {
        // Color Palette - Industrial HMI Light Theme
        public static readonly Color Background = Color.FromArgb(240, 242, 245);   // Light gray-blue background
        public static readonly Color Surface = Color.FromArgb(255, 255, 255);      // White cards/panels
        public static readonly Color SurfaceLight = Color.FromArgb(250, 251, 252); // Very light gray for inputs
        public static readonly Color Primary = Color.FromArgb(0, 120, 212);         // Blue accent (Microsoft blue)
        public static readonly Color PrimaryHover = Color.FromArgb(0, 103, 192);
        public static readonly Color PrimaryLight = Color.FromArgb(230, 240, 255); // Light blue for active states
        public static readonly Color Success = Color.FromArgb(16, 124, 16);         // Green for OK/Active
        public static readonly Color SuccessLight = Color.FromArgb(230, 245, 230);  // Light green background
        public static readonly Color Error = Color.FromArgb(232, 17, 35);         // Red for errors
        public static readonly Color ErrorLight = Color.FromArgb(255, 240, 240);  // Light red background
        public static readonly Color Warning = Color.FromArgb(255, 185, 0);       // Yellow/Orange
        public static readonly Color WarningLight = Color.FromArgb(255, 250, 230); // Light yellow background
        public static readonly Color TextPrimary = Color.FromArgb(33, 33, 33);     // Dark gray text
        public static readonly Color TextSecondary = Color.FromArgb(118, 118, 118); // Medium gray text
        public static readonly Color Border = Color.FromArgb(225, 225, 230);      // Light border gray
        public static readonly Color BorderLight = Color.FromArgb(240, 240, 245);  // Very light border
        public static readonly Color Disabled = Color.FromArgb(200, 200, 200);     // Disabled gray
        public static readonly Color Shadow = Color.FromArgb(0, 0, 0, 8);          // Subtle shadow (8% opacity)

        // Fonts
        public static Font GetDefaultFont(float size = 9f)
        {
            return new Font("Segoe UI", size, FontStyle.Regular);
        }

        public static Font GetBoldFont(float size = 9f)
        {
            return new Font("Segoe UI", size, FontStyle.Bold);
        }

        public static Font GetMonospaceFont(float size = 9f)
        {
            return new Font("Consolas", size, FontStyle.Regular);
        }

        // Apply theme to control
        public static void ApplyTheme(Control control)
        {
            if (control is Form form)
            {
                form.BackColor = Background;
                form.ForeColor = TextPrimary;
            }
            else if (control is Panel panel)
            {
                panel.BackColor = Surface;
                panel.ForeColor = TextPrimary;
            }
            else if (control is TextBox textBox)
            {
                textBox.BackColor = SurfaceLight;
                textBox.ForeColor = TextPrimary;
                textBox.BorderStyle = BorderStyle.FixedSingle;
            }
            else if (control is ComboBox comboBox)
            {
                comboBox.BackColor = SurfaceLight;
                comboBox.ForeColor = TextPrimary;
                comboBox.FlatStyle = FlatStyle.Flat;
            }
            else if (control is Button button)
            {
                button.BackColor = Primary;
                button.ForeColor = Color.White;
                button.FlatStyle = FlatStyle.Flat;
                button.FlatAppearance.BorderSize = 0;
                button.FlatAppearance.MouseOverBackColor = PrimaryHover;
                button.FlatAppearance.MouseDownBackColor = Color.FromArgb(0, 90, 158);
            }
            else if (control is Label label)
            {
                label.ForeColor = TextPrimary;
            }
            else if (control is DataGridView dgv)
            {
                dgv.BackgroundColor = Surface;
                dgv.BackColor = Surface;
                dgv.ForeColor = TextPrimary;
                dgv.GridColor = BorderLight;
                dgv.DefaultCellStyle.BackColor = Color.White;
                dgv.DefaultCellStyle.ForeColor = TextPrimary;
                dgv.DefaultCellStyle.SelectionBackColor = PrimaryLight;
                dgv.DefaultCellStyle.SelectionForeColor = TextPrimary;
                dgv.ColumnHeadersDefaultCellStyle.BackColor = Color.FromArgb(248, 249, 250);
                dgv.ColumnHeadersDefaultCellStyle.ForeColor = TextPrimary;
                dgv.ColumnHeadersDefaultCellStyle.Font = new Font(GetDefaultFont().FontFamily, 9f, FontStyle.Bold);
                dgv.RowHeadersDefaultCellStyle.BackColor = Color.FromArgb(248, 249, 250);
                dgv.RowHeadersDefaultCellStyle.ForeColor = TextPrimary;
                dgv.BorderStyle = BorderStyle.FixedSingle;
                dgv.CellBorderStyle = DataGridViewCellBorderStyle.SingleHorizontal;
            }
            else if (control is ListBox listBox)
            {
                listBox.BackColor = SurfaceLight;
                listBox.ForeColor = TextPrimary;
                listBox.BorderStyle = BorderStyle.FixedSingle;
            }
            else if (control is RichTextBox richTextBox)
            {
                richTextBox.BackColor = SurfaceLight;
                richTextBox.ForeColor = TextPrimary;
                richTextBox.BorderStyle = BorderStyle.FixedSingle;
            }
            else if (control is TabControl tabControl)
            {
                tabControl.BackColor = Background;
                tabControl.ForeColor = TextPrimary;
                tabControl.Appearance = TabAppearance.FlatButtons;
            }
            else if (control is TabPage tabPage)
            {
                tabPage.BackColor = Background;
                tabPage.ForeColor = TextPrimary;
                tabPage.Padding = new Padding(12);
            }
            else if (control is CheckBox checkBox)
            {
                checkBox.ForeColor = TextPrimary;
            }
            else if (control is NumericUpDown numericUpDown)
            {
                numericUpDown.BackColor = SurfaceLight;
                numericUpDown.ForeColor = TextPrimary;
                numericUpDown.BorderStyle = BorderStyle.FixedSingle;
            }
            else if (control is GroupBox groupBox)
            {
                groupBox.BackColor = Surface;
                groupBox.ForeColor = TextPrimary;
                groupBox.FlatStyle = FlatStyle.Flat;
            }

            // Recursively apply to children
            foreach (Control child in control.Controls)
            {
                ApplyTheme(child);
            }
        }

        // Create a card-style panel (like industrial HMI cards)
        public static Panel CreateCardPanel(string title = "", int padding = 16)
        {
            var panel = new Panel
            {
                BackColor = Surface,
                ForeColor = TextPrimary,
                Padding = new Padding(padding),
                BorderStyle = BorderStyle.FixedSingle
            };
            
            // Add subtle border color
            panel.Paint += (s, e) =>
            {
                var rect = panel.ClientRectangle;
                rect.Width -= 1;
                rect.Height -= 1;
                using (var pen = new Pen(Border, 1))
                {
                    e.Graphics.DrawRectangle(pen, rect);
                }
            };

            if (!string.IsNullOrEmpty(title))
            {
                var titleLabel = new Label
                {
                    Text = title,
                    Font = GetBoldFont(11f),
                    ForeColor = TextPrimary,
                    AutoSize = true,
                    Location = new Point(padding, padding)
                };
                panel.Controls.Add(titleLabel);
            }

            return panel;
        }

        // Create styled button (industrial HMI style)
        public static Button CreateButton(string text, Color? backColor = null, Color? foreColor = null, int height = 32)
        {
            var button = new Button
            {
                Text = text,
                BackColor = backColor ?? Primary,
                ForeColor = foreColor ?? Color.White,
                FlatStyle = FlatStyle.Flat,
                Font = GetDefaultFont(9.5f),
                Height = height,
                Padding = new Padding(16, 0, 16, 0),
                Cursor = Cursors.Hand
            };
            button.FlatAppearance.BorderSize = 0;
            button.FlatAppearance.MouseOverBackColor = backColor ?? PrimaryHover;
            button.FlatAppearance.MouseDownBackColor = Color.FromArgb(
                Math.Max(0, (backColor ?? Primary).R - 20),
                Math.Max(0, (backColor ?? Primary).G - 20),
                Math.Max(0, (backColor ?? Primary).B - 20)
            );
            return button;
        }

        // Create styled textbox
        public static TextBox CreateTextBox(string name = "", string placeholder = "")
        {
            var textBox = new TextBox
            {
                Name = name,
                BackColor = SurfaceLight,
                ForeColor = TextPrimary,
                BorderStyle = BorderStyle.FixedSingle,
                Font = GetDefaultFont(9f),
                Height = 23
            };
            return textBox;
        }

        // Create styled label
        public static Label CreateLabel(string text, FontStyle style = FontStyle.Regular, float size = 9f)
        {
            return new Label
            {
                Text = text,
                ForeColor = TextPrimary,
                Font = new Font("Segoe UI", size, style),
                AutoSize = true
            };
        }

        // Create status label
        public static Label CreateStatusLabel(string name, string initialText = "Disconnected")
        {
            var label = new Label
            {
                Name = name,
                Text = initialText,
                ForeColor = Error,
                Font = GetBoldFont(10f),
                AutoSize = true
            };
            return label;
        }

        // Update status label color
        public static void UpdateStatusLabel(Label label, bool isConnected)
        {
            if (isConnected)
            {
                label.ForeColor = Success;
                label.Text = "Connected";
            }
            else
            {
                label.ForeColor = Error;
                label.Text = "Disconnected";
            }
        }
    }
}
