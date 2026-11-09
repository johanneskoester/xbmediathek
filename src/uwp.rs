use std::{cell::RefCell, rc::Rc};
use windows::{
    self as Windows,
    core::*,
    ApplicationModel::Activation::LaunchActivatedEventArgs,
    Foundation::{TypedEventHandler, Uri},
    System::VirtualKey,
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
    UI::{
        Color,
        Core::{CoreWindow, KeyEventArgs},
        Xaml::{
            Application, ApplicationInitializationCallback, ApplicationRequiresPointerMode,
            Automation::AutomationProperties,
            Controls::{Button, Control, Grid, Image, Orientation, Page, StackPanel, TextBlock},
            FocusState, HorizontalAlignment,
            Media::Imaging::SvgImageSource,
            Media::{Brush, ImageSource, SolidColorBrush, Stretch},
            ResourceDictionary, RoutedEventHandler, Thickness, UIElement, VerticalAlignment,
            Visibility, Window,
        },
    },
};
use xbmediathek::Provider;

mod bindings {
    windows::core::include_bindings!();
}

use bindings::{
    Microsoft::UI::Xaml::Controls::{WebView2, XamlControlsResources},
    Windows::Foundation::Uri as WebUri,
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
        let background = SolidColorBrush::CreateInstanceWithColor(Color {
            A: 255,
            R: 32,
            G: 32,
            B: 32,
        })?;
        page.SetBackground(background.cast::<Brush>()?)?;
        let root = Grid::new()?;
        let picker = StackPanel::new()?;
        picker.SetOrientation(Orientation::Vertical)?;
        picker.SetSpacing(32.0)?;
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

        let transparent = SolidColorBrush::CreateInstanceWithColor(Color {
            A: 0,
            R: 0,
            G: 0,
            B: 0,
        })?;
        let active = Rc::new(RefCell::new(None::<WebView2>));
        let mut first_button = None;
        for providers in Provider::ALL.chunks(5) {
            let row = StackPanel::new()?;
            row.SetOrientation(Orientation::Horizontal)?;
            row.SetSpacing(24.0)?;
            row.SetHorizontalAlignment(HorizontalAlignment::Center)?;

            for provider in providers {
                let button = Button::new()?;
                button.SetWidth(256.0)?;
                button.SetHeight(144.0)?;
                button.SetBorderThickness(Thickness {
                    Left: 0.0,
                    Top: 0.0,
                    Right: 0.0,
                    Bottom: 0.0,
                })?;
                button.SetBorderBrush(transparent.cast::<Brush>()?)?;
                button.SetBackground(transparent.cast::<Brush>()?)?;
                button.SetFocusVisualPrimaryThickness(Thickness {
                    Left: 0.0,
                    Top: 0.0,
                    Right: 0.0,
                    Bottom: 0.0,
                })?;
                button.SetFocusVisualSecondaryThickness(Thickness {
                    Left: 0.0,
                    Top: 0.0,
                    Right: 0.0,
                    Bottom: 0.0,
                })?;
                button.SetOpacity(0.55)?;
                AutomationProperties::SetName(&button, provider.name())?;
                let logo = Image::new()?;
                logo.SetWidth(224.0)?;
                logo.SetHeight(112.0)?;
                logo.SetStretch(Stretch::Uniform)?;
                let image_source =
                    SvgImageSource::CreateInstanceWithUriSource(Uri::CreateUri(provider.logo())?)?;
                logo.SetSource(image_source.cast::<ImageSource>()?)?;
                button.SetContent(logo)?;

                button.GotFocus(RoutedEventHandler::new(move |sender, _| {
                    if let Some(sender) = sender {
                        sender.cast::<UIElement>()?.SetOpacity(1.0)?;
                    }
                    Ok(())
                }))?;
                button.LostFocus(RoutedEventHandler::new(move |sender, _| {
                    if let Some(sender) = sender {
                        sender.cast::<UIElement>()?.SetOpacity(0.55)?;
                    }
                    Ok(())
                }))?;

                let browser_root = root.clone();
                let provider_list = picker.clone();
                let browser_state = active.clone();
                button.Click(RoutedEventHandler::new(move |_, _| {
                    let browser = WebView2::new()?;
                    let element: UIElement = browser.cast()?;
                    browser.SetSource(WebUri::CreateUri(provider.url())?)?;
                    browser_root.Children()?.Append(&element)?;
                    provider_list.SetVisibility(Visibility::Collapsed)?;
                    browser.cast::<Control>()?.Focus(FocusState::Programmatic)?;
                    *browser_state.borrow_mut() = Some(browser);
                    Ok(())
                }))?;
                if first_button.is_none() {
                    first_button = Some(button.clone());
                }
                row.Children()?.Append(button)?;
            }
            picker.Children()?.Append(row)?;
        }

        root.Children()?.Append(&picker)?;
        page.SetContent(&root)?;
        let window = Window::Current()?;
        window.SetContent(page)?;
        let first_button = first_button.expect("at least one provider");
        let back_focus_button = first_button.clone();
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
                            back_focus_button.Focus(FocusState::Programmatic)?;
                        }
                        args.SetHandled(true)?;
                    }
                    Ok(())
                },
            ))?;
        window.Activate()?;
        first_button.Focus(FocusState::Programmatic)?;
        Ok(())
    }
}

pub fn run() -> Result<()> {
    unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)? };
    Application::Start(ApplicationInitializationCallback::new(|_| {
        App.new().map(|_| ())
    }))
}
