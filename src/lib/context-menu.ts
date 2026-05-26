// Disable the webview's native browser context menu.
// A custom ACMind context menu can be attached here later.
export function disableNativeContextMenu() {
	document.addEventListener("contextmenu", (event) => {
		event.preventDefault();
	});
}
