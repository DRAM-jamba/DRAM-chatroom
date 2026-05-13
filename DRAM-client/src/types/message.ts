export type Message = {
  authorUsername: string;
  content: string;
  timestamp: string;
  date: string;
  id: string;
  system?: boolean;  // true for join/leave notifications
};

export type Member = {
  username: string;
  online: boolean;
};

