// ACMind Token Reader — Content script that runs on ACMind pages.
// Reads the JWT token from localStorage and sends it to the extension storage.
// Runs silently on any page; only acts if acmind_token is found.

(function () {
  const TOKEN_KEY = "acmind_token";
  const token = localStorage.getItem(TOKEN_KEY);
  if (token) {
    // Extract API URL from current page origin
    const apiUrl = window.location.origin;
    chrome.storage.local.get(["token"], (result) => {
      // Only update if token changed
      if (result.token !== token) {
        chrome.storage.local.set({
          token: token,
          apiUrl: apiUrl,
          username: "", // will be resolved by popup/background
        });
      }
    });
  }
})();
