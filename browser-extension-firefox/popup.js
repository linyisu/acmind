// ACMind VJudge Importer — Popup Script.

const $ = (sel) => document.querySelector(sel);

// ---- State ----
async function getState() {
  const result = await chrome.storage.local.get(["apiUrl", "token", "username"]);
  return {
    apiUrl: (result.apiUrl || "").replace(/\/+$/, ""),
    token: result.token || "",
    username: result.username || "",
  };
}

async function saveState(updates) {
  await chrome.storage.local.set(updates);
}

async function clearState() {
  await chrome.storage.local.remove(["apiUrl", "token", "username"]);
}

// ---- UI transitions ----
function showLoggedIn(username) {
  $("#login-form").classList.add("hidden");
  $("#logged-in").classList.remove("hidden");
  $("#username").textContent = username;
}

function showLoginForm() {
  $("#login-form").classList.remove("hidden");
  $("#logged-in").classList.add("hidden");
  $("#login-error").classList.add("hidden");
}

function setStatus(state, text) {
  const dot = $("#status-dot");
  const label = $("#status-label");
  dot.className = "status-dot";
  if (state === "connected") dot.classList.add("connected");
  else if (state === "disconnected") dot.classList.add("disconnected");
  label.textContent = text;
}

function showError(msg) {
  const el = $("#login-error");
  el.textContent = msg;
  el.classList.remove("hidden");
}

// ---- Connection check ----
async function checkConnection() {
  const { apiUrl, token } = await getState();
  if (!apiUrl) {
    setStatus("disconnected", "No API URL configured");
    return false;
  }

  try {
    const resp = await fetch(`${apiUrl}/health`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    if (!resp.ok) {
      setStatus("disconnected", `Server returned ${resp.status}`);
      return false;
    }
  } catch {
    setStatus("disconnected", "Cannot reach ACMind API");
    return false;
  }

  // Verify token is still valid
  if (token) {
    try {
      const resp = await fetch(`${apiUrl}/api/v1/auth/me`, {
        headers: { Authorization: `Bearer ${token}` },
        signal: AbortSignal.timeout(5000),
      });
      if (resp.ok) {
        const user = await resp.json();
        setStatus("connected", `Logged in as ${user.username}`);
        return true;
      }
      // Token expired or invalid — clear it
      await clearState();
      setStatus("disconnected", "Session expired, please login again");
      showLoginForm();
      return false;
    } catch {
      setStatus("disconnected", "Cannot verify token");
      return false;
    }
  }

  setStatus("disconnected", "Not logged in");
  return false;
}

// ---- Login ----
async function login() {
  const apiUrl = $("#api-url").value.trim().replace(/\/+$/, "");
  const username = $("#username-input").value.trim();
  const password = $("#password-input").value;

  if (!apiUrl || !username || !password) {
    showError("All fields are required");
    return;
  }

  $("#btn-login").disabled = true;
  $("#login-error").classList.add("hidden");

  try {
    const resp = await fetch(`${apiUrl}/api/v1/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
      signal: AbortSignal.timeout(10000),
    });

    if (!resp.ok) {
      const err = await resp.json().catch(() => null);
      const msg = err?.error?.message || `Login failed (${resp.status})`;
      showError(msg);
      return;
    }

    const data = await resp.json();
    await saveState({
      apiUrl,
      token: data.token,
      username: data.user.username,
    });

    setStatus("connected", `Logged in as ${data.user.username}`);
    showLoggedIn(data.user.username);
  } catch (err) {
    showError("Cannot reach server. Check the API URL.");
  } finally {
    $("#btn-login").disabled = false;
  }
}

// ---- Logout ----
async function logout() {
  await clearState();
  showLoginForm();
  setStatus("disconnected", "Logged out");
}

// ---- Resolve username from token ----
async function resolveUsername() {
  const { apiUrl, token } = await getState();
  if (!apiUrl || !token) return false;

  try {
    const resp = await fetch(`${apiUrl}/api/v1/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
      signal: AbortSignal.timeout(5000),
    });
    if (resp.ok) {
      const user = await resp.json();
      await saveState({ username: user.username });
      showLoggedIn(user.username);
      setStatus("connected", `Logged in as ${user.username}`);
      return true;
    }
  } catch {}
  return false;
}

// ---- Init ----
(async () => {
  const { apiUrl, token, username } = await getState();

  // Pre-fill API URL
  if (apiUrl) {
    $("#api-url").value = apiUrl;
  } else {
    $("#api-url").value = "http://localhost:8080";
  }

  if (token && username) {
    showLoggedIn(username);
    await checkConnection();
  } else if (token && !username) {
    // Token auto-detected from ACMind page, resolve username
    setStatus("checking", "Verifying...");
    const resolved = await resolveUsername();
    if (!resolved) {
      showLoginForm();
      setStatus("disconnected", "Token expired, please login");
    }
  } else {
    showLoginForm();
    if (apiUrl) {
      await checkConnection();
    } else {
      setStatus("disconnected", "Configure API URL and login");
    }
  }
})();

// ---- Event listeners ----
$("#btn-login").addEventListener("click", login);
$("#btn-logout").addEventListener("click", logout);
$("#btn-check").addEventListener("click", checkConnection);

// Enter key submits login
$("#password-input").addEventListener("keydown", (e) => {
  if (e.key === "Enter") login();
});
