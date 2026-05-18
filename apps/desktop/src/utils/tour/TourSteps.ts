export type TourId = "main" | "settings";

export type TourStep = {
  id: string;
  route?: string;
  target: string;
  spotlightTarget?: string;
  scrollTarget?: string;
  placement?: "top" | "bottom" | "left" | "right" | "auto";
  title: string;
  body: string;
};

const tabs = {
  dashboard: "/tab/builtin-1-dashboard",
  timeline: "/tab/builtin-2-timeline",
  goals: "/tab/builtin-3-goals",
  projects: "/tab/builtin-4-projects",
  insights: "/tab/builtin-5-manifest",
};

export const TOUR_STEPS: Record<TourId, TourStep[]> = {
  main: [
    {
      id: "dashboard-overview",
      route: tabs.dashboard,
      target: "dashboard.summary",
      title: "Dashboard overview",
      body: "The dashboard summarizes tracked time for the selected date range.",
    },
    {
      id: "dashboard-widgets",
      route: tabs.dashboard,
      target: "dashboard.widgets",
      title: "Dashboard stats",
      body: "These widgets break time down by projects, apps, languages, categories, entities, and activity.",
    },
    {
      id: "goals-create",
      route: tabs.goals,
      target: "goals.new",
      title: "Set goals",
      body: "Use New Goal to create targets by time span, app, category, and excluded days.",
    },
    {
      id: "insights-year",
      route: tabs.insights,
      target: "insights.year",
      title: "Insights",
      body: "Pick a year to view long-term stats like averages, total time, top projects, top languages, and most active day.",
    },
    {
      id: "projects-list",
      route: tabs.projects,
      target: "projects.list",
      title: "Projects",
      body: "Open any project to drill into project-specific stats.",
    },
    {
      id: "timeline-controls",
      route: tabs.timeline,
      target: "timeline.controls",
      title: "Timeline",
      body: "Use these controls to choose the time range, grouping, and whether AFK events are visible.",
    },
  ],
  settings: [
    {
      id: "tracked-apps",
      route: "/settings/general",
      target: "settings.trackedApps",
      title: "Tracked apps",
      body: "Choose which currently open apps Skopio should track.",
    },
  ],
};
