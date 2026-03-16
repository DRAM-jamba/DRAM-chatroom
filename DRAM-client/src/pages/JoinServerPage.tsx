import { FormEvent, useMemo, useState } from "react";
import { ConnectionData } from "../types/app";

type JoinServerPageProps = {
  onJoinServer: (data: ConnectionData) => void;
};

function JoinServerPage({ onJoinServer }: JoinServerPageProps) {
  const [serverAddress, setServerAddress] = useState("");
  const [port, setPort] = useState("8080");
  const [displayName, setDisplayName] = useState("");
  const [statusMessage, setStatusMessage] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isIpOrDomainValid = useMemo(() => {
    if (!serverAddress.trim()) return false;

    const value = serverAddress.trim();

    const ipPattern =
      /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}$/;

    const domainPattern = /^(localhost|([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})$/;

    return ipPattern.test(value) || domainPattern.test(value);
  }, [serverAddress]);

  const isPortValid = useMemo(() => {
    const portNumber = Number(port);
    return Number.isInteger(portNumber) && portNumber >= 1 && portNumber <= 65535;
  }, [port]);

  const isFormValid =
    isIpOrDomainValid && isPortValid && displayName.trim().length >= 2;

  async function handleJoinServer(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!isFormValid) {
      setStatusMessage("Please enter a valid server address, port, and display name.");
      return;
    }

    try {
      setIsSubmitting(true);
      setStatusMessage("");

      const payload: ConnectionData = {
        serverAddress: serverAddress.trim(),
        port: Number(port),
        displayName: displayName.trim(),
      };

      await new Promise((resolve) => setTimeout(resolve, 500));

      onJoinServer(payload);
    } catch (error) {
      console.error(error);
      setStatusMessage("Failed to continue. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="app-shell">
      <section className="background-glow background-glow-left" />
      <section className="background-glow background-glow-right" />

      <div className="join-card">
        <div className="join-card__header">
          <div className="join-card__badge">DRAM Chatroom</div>
          <h1>Join Server</h1>
          <p>
            Connect to a chat server by entering its address, port, and your display name.
          </p>
        </div>

        <form className="join-form" onSubmit={handleJoinServer}>
          <div className="form-group">
            <label htmlFor="serverAddress">Server Address</label>
            <input
              id="serverAddress"
              type="text"
              placeholder="Example: 192.168.1.10 or localhost"
              value={serverAddress}
              onChange={(e) => setServerAddress(e.target.value)}
            />
            <span className="helper-text">
              Enter an IPv4 address, localhost, or a domain name.
            </span>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label htmlFor="port">Port</label>
              <input
                id="port"
                type="number"
                min="1"
                max="65535"
                placeholder="8080"
                value={port}
                onChange={(e) => setPort(e.target.value)}
              />
            </div>

            <div className="form-group">
              <label htmlFor="displayName">Display Name</label>
              <input
                id="displayName"
                type="text"
                placeholder="Enter your name"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
              />
            </div>
          </div>

          <div className="quick-info">
            <div className="quick-info__item">
              <span className="quick-info__label">Default Port</span>
              <strong>8080</strong>
            </div>
            <div className="quick-info__item">
              <span className="quick-info__label">Connection Type</span>
              <strong>Desktop Client</strong>
            </div>
          </div>

          <button className="join-button" type="submit" disabled={!isFormValid || isSubmitting}>
            {isSubmitting ? "Preparing..." : "Continue"}
          </button>

          {statusMessage && <div className="status-box">{statusMessage}</div>}
        </form>
      </div>
    </main>
  );
}

export default JoinServerPage;