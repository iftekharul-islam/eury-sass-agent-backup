import * as React from "react";
import { Icon } from "../Icons";
import { ToolCard } from "./ToolCard";
import { isWriteTool, type ToolActivityRecord } from "../../lib/session-store";

const READ_TOOLS = new Set(["read_file", "list_dir", "glob", "grep", "search"]);

function countLabel(count: number, singular: string, plural: string): string {
  return count === 1 ? `a ${singular}` : `${count} ${plural}`;
}

/**
 * "Ran a command, read 2 files" — what the turn actually did, in one line, so a
 * finished turn reads as prose instead of a stack of cards.
 */
export function summarizeActivities(activities: ToolActivityRecord[]): string {
  let ran = 0;
  let read = 0;
  let wrote = 0;
  let other = 0;

  for (const activity of activities) {
    if (activity.name === "run_command") ran += 1;
    else if (isWriteTool(activity.name)) wrote += 1;
    else if (activity.name === "list_dir") read += 1;
    else if (READ_TOOLS.has(activity.name)) read += 1;
    else other += 1;
  }

  const parts: string[] = [];
  if (ran > 0) parts.push(`ran ${countLabel(ran, "command", "commands")}`);
  if (read > 0) {
    const listed = activities.filter((a) => a.name === "list_dir").length;
    const readFiles = read - listed;
    if (listed > 0) parts.push(`explored ${countLabel(listed, "folder", "folders")}`);
    if (readFiles > 0) parts.push(`read ${countLabel(readFiles, "file", "files")}`);
  }
  if (wrote > 0) parts.push(`edited ${countLabel(wrote, "file", "files")}`);
  if (other > 0) parts.push(`used ${countLabel(other, "tool", "tools")}`);
  if (parts.length === 0) return "No tool calls";

  const joined = parts.join(", ");
  return joined.charAt(0).toUpperCase() + joined.slice(1);
}

export interface ToolGroupProps {
  activities: ToolActivityRecord[];
  /** Keeps the group open while the run is still working through it. */
  live: boolean;
  onViewDiff?: (activity: ToolActivityRecord) => void;
}

/**
 * A consecutive stretch of tool calls, folded into their own summary once they
 * finish. Open while the run works through them, so progress stays visible;
 * folded afterwards, so the answer is what remains on screen.
 */
export function ToolGroup({ activities, live, onViewDiff }: ToolGroupProps) {
  const [manual, setManual] = React.useState<boolean | null>(null);
  const expanded = manual ?? live;
  const failed = activities.filter((activity) => activity.status === "failed").length;

  React.useEffect(() => {
    if (live) setManual(null);
  }, [live]);

  if (activities.length === 0) return null;

  return (
    <div className={`tool-group${live ? " is-live" : ""}`}>
      <button
        type="button"
        className="tool-group-summary"
        aria-expanded={expanded}
        onClick={() => setManual(!expanded)}
      >
        <Icon name={expanded ? "chev-d" : "chev-r"} size={12} />
        <span className="tool-group-label">{summarizeActivities(activities)}</span>
        {failed > 0 && (
          <span className="tool-group-failed">
            <Icon name="x" size={11} />
            {failed} failed
          </span>
        )}
      </button>

      <div className="tool-group-reveal" data-open={expanded ? "true" : "false"}>
        <div className="tool-group-reveal-inner">
          {activities.map((activity) => (
            <ToolCard
              key={activity.id}
              activity={activity}
              onViewDiff={isWriteTool(activity.name) ? onViewDiff : undefined}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
