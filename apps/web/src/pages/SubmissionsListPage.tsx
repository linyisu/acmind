import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { problemsApi, submissionsApi } from "@/lib/api";
import type { Verdict } from "@acmind/shared";

const VERDICT_COLORS: Record<Verdict, "default" | "destructive" | "secondary" | "outline"> = {
  AC: "default",
  WA: "destructive",
  TLE: "destructive",
  RE: "destructive",
  CE: "destructive",
  PENDING: "secondary",
};

export default function SubmissionsListPage() {
  const qc = useQueryClient();
  const submissions = useQuery({
    queryKey: ["submissions"],
    queryFn: () => submissionsApi.list(),
  });
  const problems = useQuery({ queryKey: ["problems"], queryFn: () => problemsApi.list() });

  const [open, setOpen] = useState(false);
  const [problemId, setProblemId] = useState<string>("");
  const [language, setLanguage] = useState("rust");
  const [verdict, setVerdict] = useState<Verdict>("AC");
  const [code, setCode] = useState("fn main() {}");
  const [runtime, setRuntime] = useState("");
  const [memory, setMemory] = useState("");
  const [notes, setNotes] = useState("");

  const create = useMutation({
    mutationFn: () =>
      submissionsApi.create({
        problem_id: Number(problemId),
        language,
        code,
        verdict,
        runtime_ms: runtime ? Number(runtime) : undefined,
        memory_kb: memory ? Number(memory) : undefined,
        notes: notes || undefined,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["submissions"] });
      setOpen(false);
      setProblemId("");
      setCode("fn main() {}");
      setRuntime("");
      setMemory("");
      setNotes("");
    },
  });

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Submissions</CardTitle>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button disabled={!problems.data || problems.data.length === 0}>New</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>New submission</DialogTitle>
              <DialogDescription>Log a code submission for a problem.</DialogDescription>
            </DialogHeader>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (problemId) create.mutate();
              }}
              className="space-y-3"
            >
              <div className="space-y-1.5">
                <Label>Problem</Label>
                <Select value={problemId} onValueChange={setProblemId} required>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a problem" />
                  </SelectTrigger>
                  <SelectContent>
                    {problems.data?.map((p) => (
                      <SelectItem key={p.id} value={String(p.id)}>
                        {p.title}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label>Language</Label>
                  <Input value={language} onChange={(e) => setLanguage(e.target.value)} required />
                </div>
                <div className="space-y-1.5">
                  <Label>Verdict</Label>
                  <Select value={verdict} onValueChange={(v) => setVerdict(v as Verdict)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {(["AC", "WA", "TLE", "RE", "CE", "PENDING"] as Verdict[]).map((v) => (
                        <SelectItem key={v} value={v}>
                          {v}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label>Runtime (ms)</Label>
                  <Input
                    type="number"
                    value={runtime}
                    onChange={(e) => setRuntime(e.target.value)}
                  />
                </div>
                <div className="space-y-1.5">
                  <Label>Memory (KB)</Label>
                  <Input
                    type="number"
                    value={memory}
                    onChange={(e) => setMemory(e.target.value)}
                  />
                </div>
              </div>
              <div className="space-y-1.5">
                <Label>Code</Label>
                <textarea
                  className="flex min-h-32 w-full rounded-md border border-[var(--color-input)] bg-transparent px-3 py-2 text-sm font-mono shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-ring)]"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label>Notes</Label>
                <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
              </div>
              <DialogFooter>
                <Button type="button" variant="ghost" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={!problemId || create.isPending}>
                  {create.isPending ? "Creating…" : "Create"}
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </CardHeader>
      <CardContent>
        {submissions.isLoading ? (
          <p>Loading…</p>
        ) : submissions.data && submissions.data.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>When</TableHead>
                <TableHead>Problem</TableHead>
                <TableHead>Language</TableHead>
                <TableHead>Verdict</TableHead>
                <TableHead>Runtime</TableHead>
                <TableHead>Memory</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {submissions.data.map((s) => {
                const ptitle = problems.data?.find((p) => p.id === s.problem_id)?.title ?? `#${s.problem_id}`;
                return (
                  <TableRow key={s.id}>
                    <TableCell className="text-xs text-[var(--color-muted-foreground)]">
                      {new Date(s.submitted_at).toLocaleString()}
                    </TableCell>
                    <TableCell>{ptitle}</TableCell>
                    <TableCell>{s.language}</TableCell>
                    <TableCell>
                      <Badge variant={VERDICT_COLORS[s.verdict] ?? "outline"}>{s.verdict}</Badge>
                    </TableCell>
                    <TableCell>{s.runtime_ms ?? "—"} ms</TableCell>
                    <TableCell>{s.memory_kb ?? "—"} KB</TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        ) : (
          <p className="text-[var(--color-muted-foreground)]">No submissions yet.</p>
        )}
      </CardContent>
    </Card>
  );
}
