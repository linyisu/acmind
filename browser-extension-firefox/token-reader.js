// ACMind Token Reader — Content script that runs on ACMind pages.
// Reads the JWT token from localStorage and sends it to the extension storage.
// Only reads the token; does NOT set apiUrl (frontend and API are on different ports).

(function () {
  const TOKEN_KEY = "acmind_token";
  const token = localStorage.getItem(TOKEN_KEY);
  if (token) {
    chrome.storage.local.get(["token", "apiUrl"], (result) => {
      const updates = {};
      // Only update token if changed
      if (result.token !== token) {
        updates.token = token;
        updates.username = "";
      }
      // Set default API URL if not already configured
      if (!result.apiUrl) {
        updates.apiUrl = "http://localhost:8080";
      }
      if (Object.keys(updates).length > 0) {
        chrome.storage.local.set(updates);
      }
    });
  }
})();
