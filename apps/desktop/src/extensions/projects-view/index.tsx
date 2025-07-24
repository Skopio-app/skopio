import {
  Input,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  Skeleton,
} from "@skopio/ui";
import { useEffect, useState } from "react";
import { commands, PaginatedProjects, Project } from "../../types/tauri.gen";
import { useNavigate, useParams } from "react-router";
import { SearchIcon } from "lucide-react";
import z from "zod/v4";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { toast } from "sonner";

const schema = z.object({ query: z.string() });

const ProjectsView = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [cursors, setCursors] = useState<(number | null)[]>([]);
  const [totalPages, setTotalPages] = useState<number>(0);
  const [searchResults, setSearchResults] = useState<Project[]>([]);

  const limit = 15;

  const { id: tabId } = useParams();
  const navigate = useNavigate();

  const { register, watch } = useForm({
    resolver: zodResolver(schema),
    defaultValues: { query: "" },
  });

  const query = watch("query");

  const fetchData = async (page: number) => {
    setIsLoading(true);
    try {
      const after = cursors[page] ?? null;
      const res: PaginatedProjects = await commands.fetchProjects({
        after,
        limit,
      });

      setProjects(res.data);
      setCursors(res.cursors ?? null);
      setTotalPages(res.totalPages ?? 0);
    } catch (err) {
      console.error("Failed to fetch projects", err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData(currentPage);
  }, [currentPage]);

  useEffect(() => {
    const delay = setTimeout(() => {
      if (query.length > 0) {
        commands
          .searchProjects({ name: query, limit })
          .then(setSearchResults)
          .catch(toast.error);
      }
    }, 300);

    return () => clearTimeout(delay);
  }, [query]);

  const pageWindowSize = 7;
  const total = totalPages;
  const start = Math.max(0, currentPage - Math.floor(pageWindowSize / 2));
  const end = Math.min(total, start + pageWindowSize);

  if (projects.length === 0) {
    return (
      <p className="h-[300px] w-full flex items-center justify-center text-sm text-gray-500">
        No projects found
      </p>
    );
  }

  return (
    <div className="flex flex-col h-full px-4 py-4 space-y-4">
      <div className="relative w-full max-w-md">
        <SearchIcon className="absolute left-3.5 top-1/2 -translate-y-1/2 text-muted-foreground size-4" />
        <Input
          placeholder="Search projects..."
          className="pl-10"
          {...register("query")}
        />
      </div>

      <div className="flex-1 overflow-auto space-y-6">
        <ul className="divide-y divide-muted border">
          {isLoading
            ? Array.from({ length: limit }).map((_, i) => (
                <li key={i} className="p-4">
                  <Skeleton className="h-6 w-1/2" />
                </li>
              ))
            : (query.length > 0 ? searchResults : projects).map((project) => (
                <li
                  key={project.id}
                  className="p-4 hover:bg-muted/40 transition-colors hover:cursor-pointer"
                  onClick={() =>
                    navigate(`/tab/${tabId}/projects/${project.id}`)
                  }
                >
                  <h3 className="text-base font-medium break-words">
                    {project.name}
                  </h3>
                </li>
              ))}
        </ul>
      </div>

      {totalPages > 1 && (
        <div className="pt-4 mb-6">
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
                      <span className="px-2">...</span>
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

              {end < total && (
                <>
                  {end < total - 1 && (
                    <PaginationItem>
                      <span className="px-2">...</span>
                    </PaginationItem>
                  )}
                  <PaginationItem>
                    <PaginationLink onClick={() => setCurrentPage(total - 1)}>
                      {total}
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
