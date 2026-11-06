# XB Mediathek

An early Rust prototype for an Xbox media browser for ARD, ZDF and ARTE.

**Status:** This repository currently contains only a tested navigation core. It
does not yet contain a UWP application, a WebView2 host, controller input,
website integration, a deployable package, or video playback. The build task
builds the library for the current host; it does not produce an Xbox app.

## Local development

Install [Pixi](https://pixi.sh/), then run:

```sh
pixi run test
pixi run build
```

Pixi provisions Rust for Linux and Windows. GitHub Actions runs the tests and
builds the library in release mode for Windows and the experimental UWP target.
The UWP build uses a nightly Rust toolchain installed only on the CI runner;
Pixi provides the task but not that Tier 3 target's standard library. Neither
the local tasks nor CI build or package an Xbox app.

## Store validation

Store packaging cannot be tested until a UWP app, package manifest, assets and
packaging task exist. Once they do, CI should build the actual Store upload
package and validate its contents and Xbox device-family targeting. Microsoft
Store certification still requires a real submission. The optional Windows App
Certification Kit is deprecated, and its command-line tests require an active
Windows user session.

The [Store submission API](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions)
can create an in-progress submission and upload files before committing, but
that changes Partner Center state and is not a no-side-effects dry run. A
commit sends the submission for ingestion. No upload or publish automation
is currently configured or tested.

## Intended Xbox implementation

Use a UWP app with WinUI 2 and its WebView2 control. Microsoft explicitly
[announced WebView2 for Xbox](https://blogs.windows.com/msedgedev/2023/11/01/webview2-for-xbox-announcement/)
and the [WinUI 2 guide](https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/winui2)
lists Xbox as a supported platform. A WebView2 desktop executable or a desktop
MSIX is not an Xbox UWP package.

The host still needs to:

- Display a controller-accessible provider picker and open the provider URL.
- Disable Xbox mouse mode and route D-pad, A and B to navigation, activation
  and back. B should use WebView2 history before returning to the picker.
- Integrate page focus discovery and visible focus indicators with the Rust
  directional selection logic. Account for scrolling, dialogs, text input,
  frames, shadow DOM, video controls and pages that change dynamically.
- Handle WebView2 initialization failures and navigation errors; manage
  popups and untrusted pages without exposing privileged native APIs.
- Add UWP packaging, signing and deployment, and verify real video playback,
  DRM, geoblocking and all three services on an Xbox in Developer Mode.

The [`x86_64-uwp-windows-msvc` target](https://doc.rust-lang.org/rustc/platform-support.html)
is Rust Tier 3: official prebuilt Rust target libraries are not provided. CI
uses the Windows SDK and MSVC provided by the runner, and builds the target
standard library from Rust source. The UWP library build is not an executable
or a package. A WinUI 2 host, its Rust bindings and NuGet integration, a Store
identity, and a Pixi-driven packaging workflow remain to be implemented and
verified.