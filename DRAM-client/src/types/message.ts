export type Message = {
  authorUsername: string;
  content: string;
  timestamp: string; // e.g. "14:08"
  date: string;      // e.g. "28/03/2026" — used for date separators
  id: string;        // unique identifier for the message, used for React keys
};

export type Member = {
  username: string;
  online: boolean;
};

