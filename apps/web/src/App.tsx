import { useEffect } from "react";
import { useAuth } from "./lib/stores/auth";
import AppRouter from "./router";

export default function App() {
  const bootstrap = useAuth((s) => s.bootstrap);
  useEffect(() => {
    bootstrap();
  }, [bootstrap]);
  return <AppRouter />;
}
