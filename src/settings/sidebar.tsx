import { Icon, type IconName } from "./icon";

export type Section = "ring" | "trigger" | "general" | "about";

interface Entry {
  id: Section | null;
  label: string;
  icon: IconName;
  /** Shown instead of a page until its phase lands. */
  later?: string;
}

const TOP: readonly Entry[] = [
  { id: "ring", label: "Ring", icon: "ring" },
  { id: "trigger", label: "Trigger", icon: "mouse" },
  { id: "general", label: "General", icon: "gear" },
  { id: null, label: "Profiles", icon: "apps", later: "Phase 5" },
  { id: null, label: "Themes", icon: "palette", later: "Phase 6" },
];
const BOTTOM: readonly Entry[] = [{ id: "about", label: "About", icon: "info" }];

interface SidebarProps {
  active: Section;
  onSelect: (section: Section) => void;
}

/** The section list down the left of the settings window. */
export function Sidebar({ active, onSelect }: SidebarProps) {
  const item = (entry: Entry) => (
    <button
      key={entry.label}
      type="button"
      className={entry.id === active ? "nav-item is-active" : "nav-item"}
      aria-current={entry.id === active ? "page" : undefined}
      disabled={entry.id === null}
      onClick={() => {
        if (entry.id !== null) onSelect(entry.id);
      }}
    >
      <Icon name={entry.icon} />
      {entry.label}
      {entry.later !== undefined && <span className="nav-tag">{entry.later}</span>}
    </button>
  );
  return (
    <nav className="sidebar" aria-label="Settings sections">
      <div className="brand">
        <span className="brand-mark">
          <Icon name="ring" size={18} />
        </span>
        ClickHub
      </div>
      {TOP.map(item)}
      <div className="nav-spacer" />
      {BOTTOM.map(item)}
    </nav>
  );
}
