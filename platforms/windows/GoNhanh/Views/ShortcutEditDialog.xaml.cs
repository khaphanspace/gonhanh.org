using System.Windows;

namespace GoNhanh.Views;

public partial class ShortcutEditDialog : Window
{
    public string ShortcutKey => KeyInput.Text.Trim();
    public string ShortcutValue => ValueInput.Text.Trim();

    public ShortcutEditDialog(string key = "", string value = "")
    {
        InitializeComponent();
        KeyInput.Text = key;
        ValueInput.Text = value;
        Title = string.IsNullOrEmpty(key) ? "Thêm viết tắt" : "Sửa viết tắt";

        Loaded += (_, _) => KeyInput.Focus();
    }

    private void Save_Click(object sender, RoutedEventArgs e)
    {
        if (string.IsNullOrWhiteSpace(ShortcutKey))
        {
            global::System.Windows.MessageBox.Show("Vui lòng nhập viết tắt.", "Thông báo", MessageBoxButton.OK, MessageBoxImage.Warning);
            KeyInput.Focus();
            return;
        }

        if (string.IsNullOrWhiteSpace(ShortcutValue))
        {
            global::System.Windows.MessageBox.Show("Vui lòng nhập nội dung thay thế.", "Thông báo", MessageBoxButton.OK, MessageBoxImage.Warning);
            ValueInput.Focus();
            return;
        }

        DialogResult = true;
        Close();
    }

    private void Cancel_Click(object sender, RoutedEventArgs e)
    {
        DialogResult = false;
        Close();
    }
}
