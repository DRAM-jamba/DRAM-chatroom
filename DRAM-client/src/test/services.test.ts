import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { getServers, addServer, connectServer, removeServer } from "../services/serverService";
import { getSessions, createSession, addSession, forgetSession, deleteSession } from "../services/sessionService";
import { submitNickname, saveNickname, updateNickname } from "../services/nicknameService";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("serverService", () => {

  it("getServers returns the server list", async () => {
    const fakeServers = [{ id: "1", ipAddress: "192.168.1.1", name: "myserver", user_key: "abc" }];
    mockInvoke.mockResolvedValueOnce(fakeServers);
    const result = await getServers();
    expect(result).toEqual(fakeServers);
    expect(mockInvoke).toHaveBeenCalledWith("get_servers");
  });

  it("getServers returns empty array when there are no servers", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const result = await getServers();
    expect(result).toEqual([]);
  });

  it("addServer trims whitespace before calling tauri", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await addServer({ nickname: "  myname  ", ip: "  10.0.0.1:3000  " });
    expect(mockInvoke).toHaveBeenCalledWith("add_server", {
      ip: "10.0.0.1:3000",
      nickname: "myname",
    });
  });

  it("connectServer passes the ip through", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await connectServer("10.0.0.1:3000");
    expect(mockInvoke).toHaveBeenCalledWith("connect_server", { ip: "10.0.0.1:3000" });
  });

  it("removeServer calls forget_server not remove_server", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await removeServer("10.0.0.1:3000");
    expect(mockInvoke).toHaveBeenCalledWith("forget_server", { ip: "10.0.0.1:3000" });
  });

});

describe("sessionService", () => {

  it("getSessions returns sessions from tauri", async () => {
    const fakeSessions = [{ id: "s1", name: "testsession" }];
    mockInvoke.mockResolvedValueOnce(fakeSessions);
    const result = await getSessions();
    expect(result).toEqual(fakeSessions);
    expect(mockInvoke).toHaveBeenCalledWith("get_sessions");
  });

  it("getSessions returns empty array when user has no sessions", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const result = await getSessions();
    expect(result).toEqual([]);
  });

  it("addSession passes session key to tauri", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await addSession({ sessionKey: "abc-123" });
    expect(mockInvoke).toHaveBeenCalledWith("add_session", { sessionKey: "abc-123" });
  });

  it("forgetSession passes session key to tauri", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await forgetSession("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("forget_session", { sessionKey: "abc-123" });
  });

  it("deleteSession passes session key to tauri", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await deleteSession("abc-123");
    expect(mockInvoke).toHaveBeenCalledWith("delete_session", { sessionKey: "abc-123" });
  });

  it("createSession calls create_session then get_sessions", async () => {
    const generatedKey = "new-session-key-xyz";
    const fakeSessions = [{ id: generatedKey, name: "myroom" }];
    mockInvoke.mockResolvedValueOnce(generatedKey);
    mockInvoke.mockResolvedValueOnce(fakeSessions);
    const result = await createSession({ sessionName: "myroom", sessionKey: "" });
    expect(result.generatedKey).toBe(generatedKey);
    expect(mockInvoke).toHaveBeenCalledWith("create_session", { name: "myroom" });
  });

});

describe("nicknameService", () => {

  it("submitNickname uses newNickname as the param name", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await submitNickname("testuser");
    expect(mockInvoke).toHaveBeenCalledWith("set_nickname", { newNickname: "testuser" });
  });

  it("saveNickname calls save_nickname with nickname param", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await saveNickname("testuser");
    expect(mockInvoke).toHaveBeenCalledWith("save_nickname", { nickname: "testuser" });
  });

  it("updateNickname goes through submitNickname", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await updateNickname("newname");
    expect(mockInvoke).toHaveBeenCalledWith("set_nickname", { newNickname: "newname" });
  });

  it("submitNickname with empty string still calls tauri", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await submitNickname("");
    expect(mockInvoke).toHaveBeenCalledWith("set_nickname", { newNickname: "" });
  });

});