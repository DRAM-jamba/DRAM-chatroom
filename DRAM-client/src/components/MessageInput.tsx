import { useState } from "react";

type MessageInputProps = {
  currentUsername: string;
  onSend: (content: string) => void;
};

function MessageInput({ currentUsername, onSend }: MessageInputProps) {
  const [value, setValue] = useState("");

  const handleSend = () => {
    if (!value.trim()) return;
    onSend(value.trim());
    setValue("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleSend();
  };

  return (
    <div className="message-input-wrapper">
      <input
        className="message-input"
        placeholder={`chat as '${currentUsername}'`}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
      />
    </div>
  );
}

export default MessageInput;
