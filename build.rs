fn main() {
    #[cfg(windows)]
    windows::core::build!(Microsoft::UI::Xaml::Controls::{WebView2, XamlControlsResources});
}
