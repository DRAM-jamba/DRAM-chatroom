import { FormEvent, useState } from "react";

type ConnectPageProps = {
  onSubmit: (ip: string, nickname: string) => void;
};

export default function ConnectPage({ onSubmit }: ConnectPageProps) {
  const [ipAddress, setIpAddress] = useState("");
  const [nickname, setNickname] = useState("");
  const [error, setError] = useState("");

  const isValidIpAddress = (ip: string): boolean => {
    const ipv4Regex =
      /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)$/;

    return ipv4Regex.test(ip);
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();

    if (!ipAddress.trim() || !nickname.trim()) {
      setError("Please fill in both fields.");
      return;
    }

    if (!isValidIpAddress(ipAddress.trim())) {
      setError("Please enter a valid IP address.");
      return;
    }

    setError("");
    onSubmit(ipAddress.trim(), nickname.trim());
  };

  return (
    <div>
      <h1>Join Session</h1>

      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="ipAddress">IP Address:</label>
          <br />
          <input
            id="ipAddress"
            type="text"
            value={ipAddress}
            onChange={(e) => setIpAddress(e.target.value)}
          />
        </div>

        <br />

        <div>
          <label htmlFor="nickname">Nickname:</label>
          <br />
          <input
            id="nickname"
            type="text"
            value={nickname}
            onChange={(e) => setNickname(e.target.value)}
          />
        </div>

        <br />

        <button type="submit">Submit</button>
      </form>

      {error && (
        <>
          <br />
          <p>{error}</p>
        </>
      )}
    </div>
  );
}