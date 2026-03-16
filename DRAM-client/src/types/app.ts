export type ConnectionData = {
  serverAddress: string;
  port: number;
  displayName: string;
};

export type SessionItem = {
  id: string;
  name: string;
  sessionKey: string;
  role: "Owner" | "Member";
};