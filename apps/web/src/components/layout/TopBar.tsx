import { useNavigate } from "react-router-dom";
import { useAuth } from "@/lib/stores/auth";
import { Button } from "@/components/ui/button";

export default function TopBar() {
  const user = useAuth((s) => s.user);
  const logout = useAuth((s) => s.logout);
  const navigate = useNavigate();
  return (
    <header className="border-b border-border h-12 flex items-center justify-end px-4 gap-3">
      {user && <span className="text-sm text-muted-foreground">{user.username}</span>}
      <Button
        variant="ghost"
        size="sm"
        onClick={() => {
          logout();
          navigate("/login");
        }}
      >
        Logout
      </Button>
    </header>
  );
}
