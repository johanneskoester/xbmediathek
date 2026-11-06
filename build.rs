fn main() {
    #[cfg(windows)]
    windows::build!(Microsoft::UI::Xaml::Controls::{WebView2, XamlControlsResources});
}
