import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useState } from "react";
import { FileText, Plus, Calendar } from "lucide-react";

interface Report {
  id: string;
  report_type: string;
  title: string;
  content: string;
  start_date: string;
  end_date: string;
  created_at: string;
}

async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return [] as unknown as T;
  }
}

export function ReportsPage() {
  const queryClient = useQueryClient();
  const { data: reports, isLoading } = useQuery({
    queryKey: ["reports"],
    queryFn: () => api<Report[]>("list_reports"),
  });

  const now = new Date();
  const today = now.toISOString().split("T")[0];
  const weekAgo = new Date(now.getTime() - 7 * 86400000).toISOString().split("T")[0];

  const [dialogOpen, setDialogOpen] = useState(false);
  const [form, setForm] = useState({
    report_type: "weekly",
    title: "",
    start_date: weekAgo,
    end_date: today,
  });

  const genMutation = useMutation({
    mutationFn: () =>
      api("generate_report", {
        input: {
          report_type: form.report_type,
          title: form.title,
          start_date: form.start_date,
          end_date: form.end_date,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["reports"] });
      setDialogOpen(false);
    },
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Reports</h1>
          <p className="text-muted-foreground">
            {reports?.length ?? 0} reports generated
          </p>
        </div>

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Generate Report
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>Generate Report</DialogTitle>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label>Report Type</Label>
                <select
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={form.report_type}
                  onChange={(e) => setForm({ ...form, report_type: e.target.value })}
                >
                  <option value="weekly">Weekly</option>
                  <option value="monthly">Monthly</option>
                  <option value="custom">Custom</option>
                </select>
              </div>
              <div className="grid gap-2">
                <Label>Title</Label>
                <Input
                  value={form.title}
                  onChange={(e) => setForm({ ...form, title: e.target.value })}
                  placeholder="My Weekly Report"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label className="flex items-center gap-1">
                    <Calendar className="h-3 w-3" />
                    Start Date
                  </Label>
                  <Input
                    type="date"
                    value={form.start_date}
                    onChange={(e) =>
                      setForm({ ...form, start_date: e.target.value })
                    }
                  />
                </div>
                <div className="grid gap-2">
                  <Label className="flex items-center gap-1">
                    <Calendar className="h-3 w-3" />
                    End Date
                  </Label>
                  <Input
                    type="date"
                    value={form.end_date}
                    onChange={(e) =>
                      setForm({ ...form, end_date: e.target.value })
                    }
                  />
                </div>
              </div>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => setDialogOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={() => genMutation.mutate()}
                disabled={!form.title || genMutation.isPending}
              >
                Generate
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {/* Report list */}
      {isLoading ? (
        <div className="space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-32 w-full" />
          ))}
        </div>
      ) : reports && reports.length > 0 ? (
        <div className="space-y-4">
          {reports.map((report) => (
            <Card key={report.id}>
              <CardHeader>
                <CardTitle className="text-sm flex items-center gap-2">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  {report.title}
                  <span className="text-xs text-muted-foreground font-normal">
                    {report.report_type} · {report.start_date} → {report.end_date}
                  </span>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <pre className="text-sm text-muted-foreground whitespace-pre-wrap font-sans">
                  {report.content}
                </pre>
                <p className="text-xs text-muted-foreground mt-2">
                  Generated {new Date(report.created_at).toLocaleString()}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card className="border-dashed">
          <CardHeader>
            <CardTitle>No Reports Yet</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            Click <strong>Generate Report</strong> to create your first training
            report. Reports include submission stats, error breakdowns, and
            (when LLM is configured) AI-generated insights.
          </CardContent>
        </Card>
      )}
    </div>
  );
}
