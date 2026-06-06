import { create } from "zustand";
import type { User } from "@acmind/shared";
import * as authApi from "../api/auth";
import { getToken } from "../api/client";

interface AuthState {
  user: User | null;
  loading: boolean;
  bootstrap: () => Promise<void>;
  login: (username: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  logout: () => void;
}

export const useAuth = create<AuthState>((set) => ({
  user: null,
  loading: true,
  bootstrap: async () => {
    if (!getToken()) {
      set({ user: null, loading: false });
      return;
    }
    try {
      const user = await authApi.me();
      set({ user, loading: false });
    } catch {
      set({ user: null, loading: false });
    }
  },
  login: async (username, password) => {
    const r = await authApi.login({ username, password });
    set({ user: r.user });
  },
  register: async (username, email, password) => {
    await authApi.register({ username, email, password });
  },
  logout: () => {
    authApi.logout();
    set({ user: null });
  },
}));
