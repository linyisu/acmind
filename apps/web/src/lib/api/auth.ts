import { api, setToken } from "./client";
import type { LoginRequest, LoginResponse, RegisterRequest, User } from "@acmind/shared";

export async function login(req: LoginRequest): Promise<LoginResponse> {
  const r = await api.post<LoginResponse>("/api/v1/auth/login", req);
  setToken(r.token);
  return r;
}

export async function register(req: RegisterRequest): Promise<User> {
  return api.post<User>("/api/v1/auth/register", req);
}

export async function me(): Promise<User> {
  return api.get<User>("/api/v1/auth/me");
}

export function logout() {
  setToken(null);
}
