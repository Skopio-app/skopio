import {
  Input,
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  Skeleton,
} from "@skopio/ui";
import { useDeferredValue, useMemo, useState } from "react";
import { commands, PaginatedProjects, Project } from "@/types/tauri.gen";
import { useNavigate, useParams } from "react-router";
import { SearchIcon } from "lucide-react";
import z from "zod/v4";
import { useForm, useWatch } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";

const schema = z.object({ query: z.string() });
const LIMIT = 15;
const PAGE_WINDOW = 7;

const sortProjects = (list: Project[]) =>
  [...(list ?? [])].sort((a, b) => (b.lastUpdated ?? 0) - (a.lastUpdated ?? 0));

const ProjectsView = () => {
  const [currentPage, setCurrentPage] = useState<number>(0);

  const { id: tabId } = useParams();
  const navigate = useNavigate();

  const { register, control } = useForm<z.infer<typeof schema>>({
    resolver: standardSchemaResolver(schema),
    defaultValues: { query: "" },
  });

  const rawQuery = useWatch({ control, name: "query" });
  const query = useDeferredValue(rawQuery);
  const isSearching = query.trim().length > 0;

  const effectivePage = isSearching ? 0 : currentPage;

  const { data, isLoading, error } = useQuery({
    queryKey: [
      "projects",
      { query: isSearching ? query : null, page: effectivePage, limit: LIMIT },
    ],
    queryFn: async (): Promise<PaginatedProjects> => {
      const after = isSearching
        ? null
        : (data?.cursors?.[effectivePage] ?? null);
      return commands.fetchProjects({
        query: isSearching ? query : null,
        after,
        limit: LIMIT,
      });
    },
  });

  if (error) {
    toast.error((error as Error)?.message ?? "Failed to fetch projects");
  }

  const list = useMemo(() => sortProjects(data?.data ?? []), [data]);

  const totalPages = !isSearching ? (data?.totalPages ?? 0) : 0;

  const { start, end } = useMemo(() => {
    const total = totalPages;
    const start = Math.max(0, currentPage - Math.floor(PAGE_WINDOW / 2));
    const end = Math.min(total, start + PAGE_WINDOW);
    return { start, end };
  }, [currentPage, totalPages]);

  return (
    <div className="flex flex-col h-full mx-4 space-y-4">
      <div className="relative w-full pt-2">
        <SearchIcon className="absolute left-3.5 top-6.5 -translate-y-1/2 text-muted-foreground size-4" />
        <Input
          placeholder="Search projects..."
          className="pl-10 max-w-md"
          {...register("query")}
        />
      </div>

      <div className="flex-1 overflow-auto space-y-6 scroll-hidden">
        <ul className="divide-y divide-[var(--muted-foreground)] border border-[var(--muted-foreground)]">
          {isLoading ? (
            Array.from({ length: LIMIT }).map((_, i) => (
              <li key={i} className="p-4">
                <Skeleton className="h-6 w-1/2" />
              </li>
            ))
          ) : list.length === 0 ? (
            <p className="h-[300px] w-full flex items-center justify-center text-sm text-muted-foreground">
              No projects found
            </p>
          ) : (
            list.map((project) => (
              <li
                key={project.id}
                className="p-4 hover:bg-muted/40 transition-colors hover:cursor-pointer"
                onClick={() => navigate(`/tab/${tabId}/projects/${project.id}`)}
              >
                <h3 className="text-base font-medium break-words text-foreground">
                  {project.name}
                </h3>
              </li>
            ))
          )}
        </ul>
      </div>

      {!isSearching && totalPages > 1 && (
        <div className="pt-4 mb-5">
          <Pagination className="mt-auto">
            <PaginationContent>
              <PaginationItem>
                <PaginationPrevious
                  disabled={currentPage === 0}
                  onClick={() =>
                    setCurrentPage((prev) => Math.max(prev - 1, 0))
                  }
                />
              </PaginationItem>

              {start > 0 && (
                <>
                  <PaginationItem>
                    <PaginationLink onClick={() => setCurrentPage(0)}>
                      1
                    </PaginationLink>
                  </PaginationItem>
                  {start > 1 && (
                    <PaginationItem>
                      <PaginationEllipsis />
                    </PaginationItem>
                  )}
                </>
              )}

              {Array.from({ length: end - start }, (_, i) => {
                const pageIndex = start + i;
                return (
                  <PaginationItem key={pageIndex}>
                    <PaginationLink
                      isActive={pageIndex === currentPage}
                      onClick={() => setCurrentPage(pageIndex)}
                    >
                      {pageIndex + 1}
                    </PaginationLink>
                  </PaginationItem>
                );
              })}

              {end < totalPages && (
                <>
                  {end < totalPages - 1 && (
                    <PaginationItem>
                      <PaginationEllipsis />
                    </PaginationItem>
                  )}
                  <PaginationItem>
                    <PaginationLink
                      onClick={() => setCurrentPage(totalPages - 1)}
                    >
                      {totalPages}
                    </PaginationLink>
                  </PaginationItem>
                </>
              )}

              <PaginationItem>
                <PaginationNext
                  disabled={currentPage >= totalPages - 1}
                  onClick={() =>
                    setCurrentPage((prev) =>
                      prev + 1 < totalPages ? prev + 1 : prev,
                    )
                  }
                />
              </PaginationItem>
            </PaginationContent>
          </Pagination>
        </div>
      )}
    </div>
  );
};

export default ProjectsView;
