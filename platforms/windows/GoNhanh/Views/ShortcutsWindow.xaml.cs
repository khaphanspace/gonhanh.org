using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using GoNhanh.Services;
using Microsoft.Win32;

namespace GoNhanh.Views;

public partial class ShortcutsWindow : Window
{
    private readonly ShortcutsService _shortcuts;

    public ShortcutsWindow(ShortcutsService shortcuts)
    {
        InitializeComponent();
        _shortcuts = shortcuts;
        RefreshList();
    }

    private void RefreshList()
    {
        ShortcutsGrid.ItemsSource = null;
        ShortcutsGrid.ItemsSource = _shortcuts.Shortcuts;
        UpdateCount();
    }

    private void UpdateCount()
    {
        var enabled = _shortcuts.EnabledCount;
        var total = _shortcuts.TotalCount;
        CountText.Text = total == 0
            ? "Chưa có viết tắt nào"
            : $"{enabled}/{total} đang bật";
    }

    private void Add_Click(object sender, RoutedEventArgs e)
    {
        var dialog = new ShortcutEditDialog();
        dialog.Owner = this;
        if (dialog.ShowDialog() == true)
        {
            _shortcuts.Add(dialog.ShortcutKey, dialog.ShortcutValue, true);
            RefreshList();
        }
    }

    private void Edit_Click(object sender, RoutedEventArgs e)
    {
        EditSelectedItem();
    }

    private void Delete_Click(object sender, RoutedEventArgs e)
    {
        if (ShortcutsGrid.SelectedItem is ShortcutItem item)
        {
            var result = MessageBox.Show(
                $"Xóa viết tắt \"{item.Key}\"?",
                "Xác nhận",
                MessageBoxButton.YesNo,
                MessageBoxImage.Question);

            if (result == MessageBoxResult.Yes)
            {
                _shortcuts.RemoveById(item.Id);
                RefreshList();
            }
        }
    }

    private void Import_Click(object sender, RoutedEventArgs e)
    {
        var dialog = new OpenFileDialog
        {
            Filter = "Text files (*.txt)|*.txt|All files (*.*)|*.*",
            Title = "Nhập viết tắt"
        };

        if (dialog.ShowDialog() == true)
        {
            try
            {
                var content = System.IO.File.ReadAllText(dialog.FileName);
                var count = _shortcuts.Import(content);
                RefreshList();
                MessageBox.Show($"Đã nhập {count} viết tắt.", "Thành công", MessageBoxButton.OK, MessageBoxImage.Information);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Không thể nhập file: {ex.Message}", "Lỗi", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
    }

    private void Export_Click(object sender, RoutedEventArgs e)
    {
        if (_shortcuts.TotalCount == 0)
        {
            MessageBox.Show("Chưa có viết tắt nào để xuất.", "Thông báo", MessageBoxButton.OK, MessageBoxImage.Information);
            return;
        }

        var dialog = new SaveFileDialog
        {
            Filter = "Text files (*.txt)|*.txt",
            FileName = "gonhanh-shortcuts.txt",
            Title = "Xuất viết tắt"
        };

        if (dialog.ShowDialog() == true)
        {
            try
            {
                System.IO.File.WriteAllText(dialog.FileName, _shortcuts.Export());
                MessageBox.Show("Đã xuất viết tắt.", "Thành công", MessageBoxButton.OK, MessageBoxImage.Information);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Không thể xuất file: {ex.Message}", "Lỗi", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
    }

    private void Close_Click(object sender, RoutedEventArgs e)
    {
        Close();
    }

    private void ShortcutsGrid_MouseDoubleClick(object sender, MouseButtonEventArgs e)
    {
        EditSelectedItem();
    }

    private void ToggleEnabled_Click(object sender, RoutedEventArgs e)
    {
        if (sender is CheckBox { DataContext: ShortcutItem item })
        {
            _shortcuts.Update(item.Id, item.Key, item.Value, item.IsEnabled);
            UpdateCount();
        }
    }

    private void EditSelectedItem()
    {
        if (ShortcutsGrid.SelectedItem is ShortcutItem item)
        {
            var dialog = new ShortcutEditDialog(item.Key, item.Value);
            dialog.Owner = this;
            if (dialog.ShowDialog() == true)
            {
                _shortcuts.Update(item.Id, dialog.ShortcutKey, dialog.ShortcutValue, item.IsEnabled);
                RefreshList();
            }
        }
    }
}
