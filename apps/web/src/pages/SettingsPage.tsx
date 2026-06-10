import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/lib/stores/auth";
import { tagsApi } from "@/lib/api";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";

export default function SettingsPage() {
  const user = useAuth((s) => s.user);
  const qc = useQueryClient();
  const [name, setName] = useState("");
  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const create = useMutation({
    mutationFn: () => tagsApi.create({ name: name.trim() }),
    onSuccess: () => {
      setName("");
      qc.invalidateQueries({ queryKey: ["tags"] });
    },
  });
  const remove = useMutation({
    mutationFn: (id: number) => tagsApi.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["tags"] }),
  });

  return (
    <div className="space-y-6 max-w-2xl">
      <h1 className="text-2xl font-bold">设置</h1>

      <Card>
        <CardHeader>
          <CardTitle>账户</CardTitle>
          <CardDescription>当前登录用户</CardDescription>
        </CardHeader>
        <CardContent className="space-y-1">
          <p>
            <strong>用户名：</strong> {user?.username}
          </p>
          <p>
            <strong>邮箱：</strong> {user?.email}
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>标签管理</CardTitle>
          <CardDescription>管理你的标签词库。</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (name.trim()) create.mutate();
            }}
            className="flex gap-2"
          >
            <div className="flex-1 space-y-1.5">
              <Label htmlFor="tag-name">新建标签</Label>
              <Input
                id="tag-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="如 binary-search"
              />
            </div>
            <div className="flex items-end">
              <Button type="submit" disabled={!name.trim() || create.isPending}>
                添加
              </Button>
            </div>
          </form>
          {tags.isLoading ? (
            <p>加载中…</p>
          ) : tags.data && tags.data.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {tags.data.map((t) => (
                <span
                  key={t.id}
                  className="inline-flex items-center rounded-md bg-accent px-2 py-1 text-xs"
                >
                  {t.name}
                  <button
                    type="button"
                    className="ml-1 opacity-70 hover:opacity-100"
                    onClick={() => {
                      if (confirm(`确认删除标签「${t.name}」？`)) remove.mutate(t.id);
                    }}
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground">No tags yet.</p>
          )}
          {tags.data && tags.data.length > 0 && (
            <div className="text-xs text-muted-foreground">
              <Badge variant="outline">{tags.data.length} total</Badge>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
