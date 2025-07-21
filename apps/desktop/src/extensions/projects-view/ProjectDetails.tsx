import { useEffect, useMemo, useState } from "react";
import { useParams, useSearchParams } from "react-router";
import { commands, Project, ProjectQuery } from "../../types/tauri.gen";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
  Skeleton,
} from "@skopio/ui";
import {
  DATE_RANGE_LABELS,
  DateRangeType,
  getRangeDates,
  mapRangeToPreset,
} from "../../utils/dateRanges";
import { addDays, startOfDay } from "date-fns";
import { formatDuration } from "../../utils/time";
import RangeSelectionDialog from "../../components/RangeSelectionDialog";
import { usePresetFilter } from "./stores/usePresetFilter";
import BranchSelectionDialog from "./components/BranchSelectionDialog";
import { useTotalBucketedTime } from "./hooks/useTotalBucketedTime";
import LineChartSection from "./components/LineChartSection";
import CategoryChartSection from "./components/CategoryChartSection";
import CirclePackingChartSection from "./components/CirclePackingChartSection";

const ProjectDetails = () => {
  const { projectId } = useParams();
  const [project, setProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);

  const [customStart, setCustomStart] = useState<Date>(new Date());
  const [customEnd, setCustomEnd] = useState<Date>(new Date());

  const [pendingStart, setPendingStart] = useState<Date>(customStart);
  const [pendingEnd, setPendingEnd] = useState<Date>(customEnd);

  const [selectedBranches, setSelectedBranches] = useState<string[] | null>(
    null,
  );

  const [searchParams, setSearchParams] = useSearchParams();
  const paramRange = searchParams.get("range") as DateRangeType;
  const [selectedRange, setSelectedRange] = useState<DateRangeType>(
    paramRange && DATE_RANGE_LABELS.includes(paramRange)
      ? paramRange
      : DateRangeType.Today,
  );

  const isCustom = selectedRange === DateRangeType.Custom;

  useEffect(() => {
    const params = new URLSearchParams(searchParams);
    params.set("range", selectedRange);
    setSearchParams(params, { replace: true });
  }, [selectedRange]);

  useEffect(() => {
    if (!isCustom) return;
    const maxEnd = addDays(startOfDay(customStart ?? 0), 30);
    if (customEnd) {
      if (customEnd > maxEnd) setCustomEnd(maxEnd);
    }
  }, [customStart, customEnd, isCustom]);

  const [startDate, endDate] = useMemo(
    () => getRangeDates(selectedRange, customStart, customEnd),
    [selectedRange, customStart, customEnd],
  );

  useEffect(() => {
    const newPreset = mapRangeToPreset(selectedRange, startDate, endDate);
    usePresetFilter.setState({ preset: newPreset });
  }, [selectedRange, startDate, endDate]);

  const {
    total,
    loading: timeLoading,
    hasBranchData,
    branches,
  } = useTotalBucketedTime(project?.name ?? "", selectedBranches);

  const formattedDuration = formatDuration(total);

  useEffect(() => {
    const fetchProject = async () => {
      try {
        if (!projectId || Number.isNaN(projectId)) return;
        const query: ProjectQuery = {
          id: Number(projectId),
        };

        const result = await commands.fetchProject(query);
        setProject(result);
      } catch (err) {
        console.error("Failed to fetch project", err);
      } finally {
        setLoading(false);
      }
    };

    fetchProject();
  }, [projectId]);

  if (loading) {
    return <Skeleton className="p-4" />;
  }

  if (!project) {
    return <p className="p-4 text-red-500">Project not found.</p>;
  }
  return (
    <div className="p-4 space-y-6 mt-2">
      <Breadcrumb>
        <BreadcrumbList className="text-lg font-regular text-neutral-600">
          <BreadcrumbItem className="hover:text-neutral-950">
            <BreadcrumbLink href="/">Projects</BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbLink>{project.name}</BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      {!timeLoading ? (
        <p className="text-muted-foreground">
          <span className="font-medium text-gray-800">{formattedDuration}</span>{" "}
          for{" "}
          <RangeSelectionDialog
            selectedRange={selectedRange}
            setSelectedRange={setSelectedRange}
            pendingStart={pendingStart}
            setPendingStart={setPendingStart}
            setPendingEnd={setPendingEnd}
            pendingEnd={pendingEnd}
            setCustomStart={setCustomStart}
            setCustomEnd={setCustomEnd}
            isCustom={isCustom}
            searchParams={searchParams}
            setSearchParams={setSearchParams}
          />{" "}
          in <span className="text-gray-900 font-semibold">{project.name}</span>{" "}
          {hasBranchData && (
            <>
              under{" "}
              <BranchSelectionDialog
                branches={branches}
                selectedBranch={selectedBranches}
                onSelect={setSelectedBranches}
              />{" "}
              branches
            </>
          )}
        </p>
      ) : (
        <Skeleton className="h-4 max-w-76" />
      )}

      <LineChartSection projectName={project.name} />
      <CategoryChartSection projectName={project.name} />
      <CirclePackingChartSection projectName={project.name} />
    </div>
  );
};

export default ProjectDetails;
