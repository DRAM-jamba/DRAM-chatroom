// useState used to store and update data (state)
import { FormEvent, useState } from "react";

type ConnectPageProps = {
  onSubmit: (ip: string) => void;
};

export default function ConnectPage({ onSubmit }: ConnectPageProps) {
  // State to store the IP address entered by the user
  const [ipAddress, setIpAddress] = useState("");

  // State to store error messages
  const [error, setError] = useState("");

  // Function to validate if the IP address is correct
  const isValidIpAddress = (ip: string): boolean => {
    const ipv4Regex =
      /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)$/;

    return ipv4Regex.test(ip);
  };

  // Function executed when the form is submitted
  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();

    if (!ipAddress.trim()) {
      setError("Please enter an IP address.");
      return;
    }

    if (!isValidIpAddress(ipAddress.trim())) {
      setError("Please enter a valid IP address.");
      return;
    }

    setError("");
    onSubmit(ipAddress.trim());
  };

  return (
    <div>
      <h1>Connect to the Server</h1>

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