import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { Light as SyntaxHighlighter } from "react-syntax-highlighter";
import vscDarkPlus from "react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus";
import cpp from "react-syntax-highlighter/dist/esm/languages/hljs/cpp";
import python from "react-syntax-highlighter/dist/esm/languages/hljs/python";
import java from "react-syntax-highlighter/dist/esm/languages/hljs/java";
import rust from "react-syntax-highlighter/dist/esm/languages/hljs/rust";
import { problemsApi, submissionsApi, tagsApi } from "@/lib/api";
import type { Submission } from "@acmind/shared";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ExternalLink, Copy, Check } from "lucide-react";

SyntaxHighlighter.registerLanguage("cpp", cpp);
SyntaxHighlighter.registerLanguage("c", cpp);
SyntaxHighlighter.registerLanguage("python", python);
SyntaxHighlighter.registerLanguage("java", java);
SyntaxHighlighter.registerLanguage("rust", rust);

const LANG_MAP: Record<string, string> = {
  "C++": "cpp",
  "C++17": "cpp",
  "C++20": "cpp",
  "C": "c",
  "Python3": "python",
  "Python": "python",
  "Java": "java",
  "Rust": "rust",
};

function verdictColor(v: string): string {
  if (v === "AC") return "bg-green-600 text-white";
  if (v === "WA") return "bg-red-600 text-white";
  if (v === "TLE" || v === "MLE") return "bg-yellow-600 text-white";
  if (v === "RE") return "bg-orange-600 text-white";
  if (v === "CE") return "bg-gray-500 text-white";
  return "bg-secondary text-secondary-foreground";
}

export default function ProblemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedSubmission, setSelectedSubmission] = useState<Submission | null>(null);
  const [copied, setCopied] = useState(false);

  const problem = useQuery({
    queryKey: ["problems", Number(id)],
    queryFn: () => problemsApi.get(Number(id!)),
    enabled: !!id,
  });

  const submissions = useQuery({
    queryKey: ["submissions", Number(id)],
    queryFn: () => submissionsApi.list(Number(id!)),
    enabled: !!id,
  });

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });

  if (problem.isLoading) {
    return <p className="text-muted-foreground">Loading…</p>;
  }

  if (!problem.data) {
    return <p className="text-muted-foreground">Problem not found.</p>;
  }

  const p = problem.data;
  const tagNames = p.tag_ids
    .map((tid) => tags.data?.find((t) => t.id === tid)?.name)
    .filter(Boolean) as string[];

  function copyCode() {
    if (selectedSubmission?.code) {
      navigator.clipboard.writeText(selectedSubmission.code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  function getLanguageHint(lang: string): string {
    for (const [key, value] of Object.entries(LANG_MAP)) {
      if (lang.toLowerCase().includes(key.toLowerCase())) return value;
    }
    return "plaintext";
  }

  return (
    <div className="space-y-6 max-w-5xl">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">{p.title}</h1>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <span>{p.source}</span>
            {p.external_id && <span>· {p.external_id}</span>}
            {p.difficulty != null && (
              <>
                <span>·</span>
                <Badge variant="outline">Difficulty {p.difficulty}</Badge>
              </>
            )}
          </div>
        </div>
        <Button variant="outline" onClick={() => navigate(`/problems/${id}/edit`)}>
          Edit
        </Button>
      </div>

      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">{n}</Badge>
          ))}
        </div>
      )}

      {p.url && (
        <a href={p.url} target="_blank" rel="noopener noreferrer" className="inline-block mb-2">
          <Button variant="outline" size="sm" className="gap-1.5">
            <ExternalLink className="h-3.5 w-3.5" />
            View on {p.source}
          </Button>
        </a>
      )}

      {p.statement && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Statement</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="statement-content">
              <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                {p.statement}
              </Markdown>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Submissions</CardTitle>
        </CardHeader>
        <CardContent>
          {submissions.isLoading ? (
            <p className="text-muted-foreground">Loading…</p>
          ) : submissions.data && submissions.data.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>When</TableHead>
                  <TableHead>Language</TableHead>
                  <TableHead>Verdict</TableHead>
                  <TableHead>Runtime</TableHead>
                  <TableHead>Memory</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {submissions.data.map((s) => (
                  <TableRow
                    key={s.id}
                    className="cursor-pointer hover:bg-accent/50"
                    onClick={() => setSelectedSubmission(s)}
                  >
                    <TableCell className="text-xs text-muted-foreground">
                      {new Date(s.submitted_at).toLocaleString()}
                    </TableCell>
                    <TableCell>{s.language}</TableCell>
                    <TableCell>
                      <span className={`inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${verdictColor(s.verdict)}`}>
                        {s.verdict}
                      </span>
                    </TableCell>
                    <TableCell>{s.runtime_ms ?? "—"} ms</TableCell>
                    <TableCell>{s.memory_kb ?? "—"} KB</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-muted-foreground">No submissions yet.</p>
          )}
        </CardContent>
      </Card>

      {/* Submission code dialog */}
      <Dialog open={!!selectedSubmission} onOpenChange={(open) => { if (!open) setSelectedSubmission(null); }}>
        <DialogContent className="w-[95vw] max-w-[1400px] h-[90vh] p-0 gap-0 overflow-hidden flex flex-col">
          {/* Header bar */}
          <div className="flex items-center justify-between px-4 py-2.5 border-b bg-muted/30 shrink-0">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-sm">#{selectedSubmission?.id}</span>
              {selectedSubmission && (
                <span className={`inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${verdictColor(selectedSubmission.verdict)}`}>
                  {selectedSubmission.verdict}
                </span>
              )}
              {selectedSubmission && (
                <span className="text-xs text-muted-foreground">
                  {selectedSubmission.language} · {selectedSubmission.runtime_ms ?? "—"}ms · {selectedSubmission.memory_kb ?? "—"}KB
                </span>
              )}
            </div>
            <Button variant="ghost" size="sm" className="gap-1.5" onClick={copyCode}>
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
              {copied ? "Copied" : "Copy"}
            </Button>
          </div>
          {/* Code area */}
          <div className="flex-1 overflow-auto">
            {selectedSubmission?.code ? (
              <SyntaxHighlighter
                language={getLanguageHint(selectedSubmission.language)}
                style={vscDarkPlus}
                customStyle={{
                  margin: 0,
                  borderRadius: 0,
                  fontSize: "0.82rem",
                  lineHeight: "1.6",
                  minHeight: "100%",
                  background: "#1e1e1e",
                }}
                showLineNumbers
              >
                {selectedSubmission.code}
              </SyntaxHighlighter>
            ) : (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                No source code available for this submission.
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
