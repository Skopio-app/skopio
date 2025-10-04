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
import { useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import { commands, PaginatedProjects, Project } from "@/types/tauri.gen";
import { useNavigate, useParams } from "react-router";
import { SearchIcon } from "lucide-react";
import z from "zod/v4";
import { useForm } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { toast } from "sonner";

const schema = z.object({ query: z.string() });
const LIMIT = 15;
const PAGE_WINDOW = 7;

const ProjectsView = () => {
  const [list, setList] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [cursors, setCursors] = useState<(string | null)[]>([]);
  const [totalPages, setTotalPages] = useState<number>(0);

  const { id: tabId } = useParams();
  const navigate = useNavigate();

  const { register, watch } = useForm<z.infer<typeof schema>>({
    resolver: standardSchemaResolver(schema),
    defaultValues: { query: "" },
  });

  const rawQuery = watch("query");
  const query = useDeferredValue(rawQuery);
  const prevQueryRef = useRef<string>("");

  useEffect(() => {
    const controller = new AbortController();
    const run = async () => {
      try {
        setIsLoading(true);

        const searching = query.trim().length > 0;
        const after = searching ? null : (cursors[currentPage] ?? null);

        if (searching && prevQueryRef.current.trim().length === 0) {
          setCurrentPage(0);
        }

        const res: PaginatedProjects = await commands
          .fetchProjects({
            query: searching ? query : null,
            after,
            limit: LIMIT,
          })
          .catch((e) => {
            if (!controller.signal.aborted) throw e;
            return null as any;
          });

        if (controller.signal.aborted || !res) return;

        const data = (res.data ?? []).slice().sort((a, b) => {
          const la = a.lastUpdated ?? 0;
          const lb = b.lastUpdated ?? 0;
          return lb - la;
        });

        setList(data);

        if (!searching) {
          setCursors(res.cursors ?? []);
          setTotalPages(res.totalPages ?? 0);
        }
      } catch (err) {
        if (!controller.signal.aborted) {
          toast.error((err as Error)?.message ?? "Failed to fetch projects");
        }
      } finally {
        if (!controller.signal.aborted) {
          setIsLoading(false);
          prevQueryRef.current = query;
        }
      }
    };

    run();
    return () => controller.abort();
  }, [currentPage, query]);

  const { start, end } = useMemo(() => {
    const total = totalPages;
    const start = Math.max(0, currentPage - Math.floor(PAGE_WINDOW / 2));
    const end = Math.min(total, start + PAGE_WINDOW);
    return { start, end };
  }, [currentPage, totalPages]);

  const isSearching = query.trim().length > 0;

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
