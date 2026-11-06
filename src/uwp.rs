use std::{cell::RefCell, rc::Rc};
use windows::{
    self as Windows,
    core::*,
    ApplicationModel::Activation::LaunchActivatedEventArgs,
    Foundation::TypedEventHandler,
    System::VirtualKey,
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
    UI::{
        Core::{CoreWindow, KeyEventArgs},
        Xaml::{
            Application, ApplicationInitializationCallback, ApplicationRequiresPointerMode,
            Controls::{Button, Control, Grid, Orientation, Page, StackPanel, TextBlock},
            FocusState, HorizontalAlignment, ResourceDictionary, RoutedEventHandler, Thickness,
            UIElement, VerticalAlignment, Visibility, Window,
        },
    },
};
use xbmediathek::Provider;

mod bindings {
    windows::include_bindings!();
}

use bindings::{
    Microsoft::UI::Xaml::Controls::{WebView2, XamlControlsResources},
    Windows::Foundation::Uri,
};

#[implement(extend Windows::UI::Xaml::Application, override OnLaunched)]
struct App;

impl App {
    #[allow(non_snake_case)]
    fn OnLaunched(&self, _args: &Option<LaunchActivatedEventArgs>) -> Result<()> {
        let app = Application::Current()?;
        app.SetRequiresPointerMode(ApplicationRequiresPointerMode::WhenRequested)?;
        let resources: ResourceDictionary = XamlControlsResources::new()?.cast()?;
        app.Resources()?.MergedDictionaries()?.Append(resources)?;

        let page = Page::new()?;
        let root = Grid::new()?;
        let picker = StackPanel::new()?;
        picker.SetOrientation(Orientation::Vertical)?;
        picker.SetSpacing(16.0)?;
        picker.SetHorizontalAlignment(HorizontalAlignment::Center)?;
        picker.SetVerticalAlignment(VerticalAlignment::Center)?;
        picker.SetPadding(Thickness {
            Left: 32.0,
            Top: 32.0,
            Right: 32.0,
            Bottom: 32.0,
        })?;

        let heading = TextBlock::new()?;
        heading.SetText("Mediatheken")?;
        heading.SetFontSize(32.0)?;
        picker.Children()?.Append(heading)?;

        let active = Rc::new(RefCell::new(None::<WebView2>));
        let mut first_button = None;
        for provider in Provider::ALL {
            let button = Button::new()?;
            button.SetWidth(360.0)?;
            button.SetHeight(68.0)?;
            button.SetFontSize(22.0)?;
            let label = TextBlock::new()?;
            label.SetText(provider.name())?;
            button.SetContent(label)?;

            let browser_root = root.clone();
            let provider_list = picker.clone();
            let browser_state = active.clone();
            button.Click(RoutedEventHandler::new(move |_, _| {
                let browser = WebView2::new()?;
                let element: UIElement = browser.cast()?;
                browser.SetSource(Uri::CreateUri(provider.url())?)?;
                browser_root.Children()?.Append(&element)?;
                provider_list.SetVisibility(Visibility::Collapsed)?;
                browser.cast::<Control>()?.Focus(FocusState::Programmatic)?;
                *browser_state.borrow_mut() = Some(browser);
                Ok(())
            }))?;
            if first_button.is_none() {
                first_button = Some(button.clone());
            }
            picker.Children()?.Append(button)?;
        }

        root.Children()?.Append(&picker)?;
        page.SetContent(&root)?;
        let window = Window::Current()?;
        window.SetContent(page)?;
        let first_button = first_button.expect("at least one provider");
        let current_picker = picker.clone();
        let current_root = root.clone();
        window
            .CoreWindow()?
            .KeyDown(TypedEventHandler::<CoreWindow, KeyEventArgs>::new(
                move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    if args.VirtualKey()? != VirtualKey::GamepadB
                        && args.VirtualKey()? != VirtualKey::Escape
                    {
                        return Ok(());
                    }
                    let mut current = active.borrow_mut();
                    if let Some(browser) = current.as_ref() {
                        if browser.CanGoBack()? {
                            browser.GoBack()?;
                        } else {
                            browser.Close()?;
                            current_root.Children()?.RemoveAt(1)?;
                            *current = None;
                            current_picker.SetVisibility(Visibility::Visible)?;
                            first_button.Focus(FocusState::Programmatic)?;
                        }
                        args.SetHandled(true)?;
                    }
                    Ok(())
                },
            ))?;
        window.Activate()?;
        picker
            .Children()?
            .GetAt(1)?
            .cast::<Button>()?
            .Focus(FocusState::Programmatic)?;
        Ok(())
    }
}

pub fn run() -> Result<()> {
    unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)? };
    Application::Start(ApplicationInitializationCallback::new(|_| {
        App.new().map(|_| ())
    }))
}
