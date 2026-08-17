import React, { useState, useEffect } from "react";
import { Icon } from "./Icons";
import { EuryMark } from "./EuryMark";
import { generateCodeChallenge, generateCodeVerifier, authIpc } from "../lib/auth";
import { getAgentApiUrl, getAgentAuthorizeUrl } from "../lib/config";
import { openExternalUrl } from "../lib/open";

export interface LoginModalProps {
  onLoginSuccess: () => void;
}

export const LoginModal: React.FC<LoginModalProps> = ({ onLoginSuccess }) => {
  const [step, setStep] = useState<"start" | "loading" | "polling" | "success">("start");
  const [userCode, setUserCode] = useState<string | null>(null);
  const [deviceCode, setDeviceCode] = useState<string | null>(null);
  const [codeVerifier, setCodeVerifier] = useState<string | null>(null);
  const [verificationUri, setVerificationUri] = useState<string>(getAgentAuthorizeUrl());
  const [error, setError] = useState<string | null>(null);
  const [isCopied, setIsCopied] = useState(false);
  const [isOfflineBackend, setIsOfflineBackend] = useState(false);

  const startDeviceFlow = async () => {
    setError(null);
    setIsOfflineBackend(false);
    setStep("loading");

    try {
      const verifier = generateCodeVerifier();
      setCodeVerifier(verifier);
      const challenge = await generateCodeChallenge(verifier);

      const response = await fetch(`${getAgentApiUrl()}/agent/v1/auth/device/start`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          codeChallenge: challenge,
          deviceName: "Eury Agent Desktop",
          platform: "darwin",
          appVersion: "0.1.0",
        }),
      });

      if (!response.ok) {
        throw new Error(`Server responded with status ${response.status}`);
      }

      const data = await response.json();
      setUserCode(data.userCode || "EURY-7842");
      setDeviceCode(data.deviceCode || "dev-sample-device-code");
      if (data.verificationUri) {
        setVerificationUri(data.verificationUri);
      }
      setStep("polling");

      const authUrl = `${data.verificationUri || getAgentAuthorizeUrl()}?user_code=${encodeURIComponent(data.userCode)}`;
      try {
        await openExternalUrl(authUrl);
      } catch (openError: unknown) {
        const openMsg = openError instanceof Error ? openError.message : "Could not open browser";
        setError(`${openMsg} Visit ${authUrl} manually and enter the code shown below.`);
      }
    } catch (e: unknown) {
      let errMsg = e instanceof Error ? e.message : "Failed to connect to authentication server";
      if (errMsg.toLowerCase().includes("load failed") || errMsg.toLowerCase().includes("failed to fetch")) {
        errMsg = `Authentication server at ${getAgentApiUrl()} is currently unreachable.`;
      }
      setError(errMsg);
      setIsOfflineBackend(true);
      setStep("start");
    }
  };

  const handleSimulatedDevAuth = async () => {
    try {
      await authIpc.setTokens({
        access_token: "demo_access_token_" + Date.now(),
        refresh_token: "demo_refresh_token_" + Date.now(),
      });
    } catch {
      // IPC fallback when running outside Tauri
    }
    setStep("success");
    setTimeout(() => {
      onLoginSuccess();
    }, 500);
  };

  const handleReopenBrowser = async () => {
    if (!userCode) return;
    const authUrl = `${verificationUri}?user_code=${encodeURIComponent(userCode)}`;
    try {
      await openExternalUrl(authUrl);
      setError(null);
    } catch (openError: unknown) {
      const openMsg = openError instanceof Error ? openError.message : "Could not open browser";
      setError(`${openMsg} Visit ${authUrl} manually.`);
    }
  };

  const handleCopyCode = () => {
    if (userCode) {
      navigator.clipboard.writeText(userCode);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    }
  };

  useEffect(() => {
    if (step !== "polling" || !deviceCode || !codeVerifier) return;

    let active = true;

    const poll = async () => {
      try {
        const response = await fetch(`${getAgentApiUrl()}/agent/v1/auth/device/poll`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            deviceCode,
            codeVerifier,
          }),
        });

        if (!active) return;

        if (response.status === 200) {
          const data = await response.json();
          if (data.status === "authorization_pending") {
            return;
          }

          if (data.accessToken && data.refreshToken) {
            await authIpc.setTokens({
              access_token: data.accessToken,
              refresh_token: data.refreshToken,
              // Both are needed to renew the session later; without them the
              // access token simply dies after ~15 minutes.
              device_id: data.deviceId,
              expires_at: Math.floor(Date.now() / 1000) + (data.expiresIn ?? 900),
            });
            setStep("success");
          }
        } else {
          const errorData = await response.json().catch(() => ({}));
          setError(errorData.message || "Device authorization expired or was rejected");
          setStep("start");
        }
      } catch (e: unknown) {
        if (!active) return;
        const errMsg = e instanceof Error ? e.message : "Connection lost during authorization";
        setError(errMsg);
        setStep("start");
      }
    };

    const timer = setInterval(poll, 4000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [step, deviceCode, codeVerifier]);

  useEffect(() => {
    if (step !== "success") return;
    const timer = window.setTimeout(() => {
      onLoginSuccess();
    }, 500);
    return () => window.clearTimeout(timer);
  }, [step, onLoginSuccess]);

  return (
    <div className="login-screen">
      <aside className="login-aside">
        <div className="login-aside-inner">
          <div className="login-aside-brand">
            <EuryMark size={40} className="login-mark" />
            <div>
              <span className="login-aside-name">Eury Agent</span>
              <span className="login-aside-tag">Desktop</span>
            </div>
          </div>

          <p className="login-aside-lead">
            Local agent for your repos — models, sandbox tools, and workspace sync through the Eury gateway.
          </p>

          <ul className="login-aside-points">
            <li>
              <Icon name="shield" size={14} />
              <span>Sandbox-isolated tool execution</span>
            </li>
            <li>
              <Icon name="code" size={14} />
              <span>Code and Home workspaces</span>
            </li>
            <li>
              <Icon name="plug" size={14} />
              <span>Device authorization (PKCE)</span>
            </li>
          </ul>
        </div>

        <div className="login-aside-foot">
          <span className="mono">v0.1.0</span>
        </div>
      </aside>

      <section className="login-main">
        <div className="login-panel">
          {step === "success" ? (
            <div className="login-success">
              <div className="login-success-icon">
                <Icon name="check" size={22} />
              </div>
              <h1>Signed in</h1>
              <p className="lead">Loading your workspace…</p>
            </div>
          ) : (
            <>
              <header className="login-header">
                <h1>Sign in</h1>
                <p className="lead">Continue with your Eury account to use the agent.</p>
              </header>

              {error && (
                <div className="login-error">
                  <div className="login-error-title">
                    <Icon name="alert" size={14} />
                    <span>Could not sign in</span>
                  </div>
                  <p>{error}</p>
                  {isOfflineBackend && (
                    <p className="login-error-hint">
                      Check that the backend is running at <span className="mono">{getAgentApiUrl()}</span>.
                    </p>
                  )}
                </div>
              )}

              {step === "start" && (
                <div className="login-actions">
                  <button type="button" className="btn primary login-continue" onClick={startDeviceFlow}>
                    <Icon name="globe" />
                    Continue in browser
                  </button>
                  <p className="login-footnote">
                    Opens your system browser for secure device authorization. No password is stored locally.
                  </p>

                  {isOfflineBackend && (
                    <div className="login-offline-actions">
                      <button type="button" className="btn" onClick={startDeviceFlow}>
                        <Icon name="rotate-ccw" size={14} />
                        Retry connection
                      </button>
                      <button type="button" className="btn ghost sm login-dev" onClick={handleSimulatedDevAuth}>
                        Dev: mock sign-in
                      </button>
                    </div>
                  )}
                </div>
              )}

              {step === "loading" && (
                <div className="login-loading">
                  <span className="spinner login-spinner" />
                  <span>Starting authorization…</span>
                </div>
              )}

              {step === "polling" && userCode && (
                <div className="login-polling">
                  <ol className="login-steps">
                    <li>
                      <span className="login-step-n">1</span>
                      <span>Your browser should open to the Eury sign-in page.</span>
                    </li>
                    <li>
                      <span className="login-step-n">2</span>
                      <span>Sign in and approve access for this device.</span>
                    </li>
                    <li>
                      <span className="login-step-n">3</span>
                      <span>Enter this code if the page asks for it.</span>
                    </li>
                  </ol>

                  <div className="login-code-box">
                    <span className="login-code-label">Verification code</span>
                    <span className="login-code-digits">{userCode}</span>
                    <button type="button" className="btn sm ghost login-copy" onClick={handleCopyCode}>
                      <Icon name={isCopied ? "check" : "copy"} size={14} />
                      {isCopied ? "Copied" : "Copy code"}
                    </button>
                  </div>

                  <div className="login-waiting">
                    <span className="spinner login-spinner-sm" />
                    <span>Waiting for approval…</span>
                  </div>

                  <div className="login-poll-actions">
                    <button type="button" className="btn sm ghost" onClick={() => setStep("start")}>
                      Cancel
                    </button>
                    <button type="button" className="btn sm" onClick={handleReopenBrowser}>
                      <Icon name="globe" size={14} />
                      Open browser again
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </section>
    </div>
  );
};
