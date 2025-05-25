import { HeroPattern } from "@/components/HeroPattern";
import clsx from "clsx";
import { useScroll, useTransform, motion } from "framer-motion";
import Link from "next/link";
import ProfileMenu from "@/components/ProfileMenu";
import {Cog6ToothIcon, HomeIcon, InformationCircleIcon, PlayCircleIcon, KeyIcon} from "@heroicons/react/24/outline";
import {useQuery} from "@tanstack/react-query";
import {axiosInstance} from "@/utils/axiosUtils";
import useIsAuthenticated from "@/hooks/useIsAuthenticated";
import {useParams, usePathname} from "next/navigation";
import {api} from "@/lib/api";

export default function ConsoleLayout({ children }) {
  useIsAuthenticated(false);

  let { scrollY } = useScroll();
  let bgOpacityLight = useTransform(scrollY, [0, 72], [0.5, 0.9]);
  let bgOpacityDark = useTransform(scrollY, [0, 72], [0.2, 0.8]);

  const pathname = usePathname();
  const params = useParams();

  const userQuery = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const response = await api.get("/user");
      return await response.json();
    },
  });
  const infoQuery = useQuery({
    queryKey: ['info'],
    queryFn: async () => {
      const response = await api.get("/info");
      return await response.json();
    },
  });

  if (userQuery.isLoading || !userQuery.data) return null;
  if (infoQuery.isLoading || !infoQuery.data) return null;

  const navigation = [
    { name: "Overview", icon: HomeIcon, href: `/${userQuery.data.name}` },
    { name: "Certificates", icon: KeyIcon, href: `/${userQuery.data.name}/certificates` },
    { name: "Account", icon: InformationCircleIcon, href: `/account` },
  ];

  const navigationService = [
    { name: "Overview", icon: HomeIcon, href: `/${userQuery.data.name}/${params.serviceName}` },
    // { name: "Deployments", icon: PlayCircleIcon, href: `/${userQuery.data.name}/${params.serviceName}` },
    // { name: "Cron Jobs", icon: ArrowPathIcon, href: `/${userQuery.data.username}/${router.query.projectName}/cron-jobs` },
    // { name: "Integrations", icon: ArrowsRightLeftIcon, href: `/${userQuery.data.username}/${router.query.projectName}/integrations` },
    // { name: "Analytics", icon: ChartBarIcon, href: `/${userQuery.data.username}/${router.query.projectName}/analytics` },
    // { name: "Logs", icon: DocumentTextIcon, href: `/${userQuery.data.username}/${router.query.projectName}/logs` },
    // { name: "Settings", icon: Cog6ToothIcon, href: `/${userQuery.data.username}/${params.serviceName}/settings` },
  ];

  const isServicePage = params.username && params.serviceName;

  const isCurrentPath = (path, href) => {
    // Exact match
    if (path === href) {
      return true;
    }

    // For overview/home pages, don't match if there are additional segments
    if (href === `/${userQuery.data.name}`) {
      return path === href;
    }

    // For service overview page, don't match if there are additional segments
    if (href === `/${userQuery.data.name}/${params.serviceName}`) {
      return path === href;
    }

    // For any other links, check if the current path starts with the href
    // and has a slash at the href length position (indicating a sub-path)
    if (path.startsWith(href)) {
      // If there's a character after href, it should be a slash
      if (path.length === href.length || path.charAt(href.length) === '/') {
        return true;
      }
    }

    return false;
  };

  const currentNavigation = () => {
    if (isServicePage) {
      return navigationService;
    }
    return navigation
  };

  return (
    <>
      <title>Dashboard - Dosei</title>
      <div className="flex flex-col min-h-screen antialiased">
        <HeroPattern/>
        <main className="flex flex-col flex-1 bg-white/5">
          <motion.nav
            className={clsx(
              "transition z-10 fixed w-screen border-b border-b-white/10",
              // colors
              'backdrop-blur-sm dark:backdrop-blur',
              'bg-white/[var(--bg-opacity-light)] dark:bg-zinc-900/[var(--bg-opacity-dark)]'
            )}
            style={{
              '--bg-opacity-light': bgOpacityLight,
              '--bg-opacity-dark': bgOpacityDark,
            }}
          >
            <div
              className={clsx(
                'absolute inset-x-0 top-full h-px transition',
                'bg-zinc-900/7.5 dark:bg-white/7.5',
              )}
            />
            <div className="mx-auto px-2 sm:px-6 lg:px-8">
              <div className="relative flex justify-between pt-2.5">
                <div className="flex-1 flex items-stretch justify-start">
                  <div className="flex-shrink-0 flex items-center">
                    <Link href="/">
                      <picture>
                        <img src="/logo-no-text.svg" className="h-8 w-auto block dark:hidden" alt="Dosei Logo"/>
                        <img src="/logo-no-text-white.svg" className="h-8 w-auto hidden dark:block" alt="Dosei Logo"/>
                      </picture>
                    </Link>
                    <div className="ml-2 space-x-2">
                      <Link href={`/${userQuery.data.name}`} className={clsx(
                        (isServicePage) ? "" : "font-semibold",
                        "hover:bg-gray-100 px-2.5 py-1.5 rounded-md inline-flex hover:text-gray-900",
                        "dark:hover:bg-neutral-900 dark:text-white dark:hover:text-neutral-100"
                      )}>
                        {userQuery.data?.name.toLowerCase()}
                      </Link>
                      {
                        (isServicePage) &&
                        <>
                          <span className="dark:text-white">/</span>
                          <Link
                            className={clsx(
                              "font-semibold",
                              "hover:bg-gray-100 px-2.5 py-1.5 rounded-md inline-flex hover:text-gray-900",
                              "dark:hover:bg-neutral-900 dark:text-white dark:hover:text-neutral-100"
                            )}
                            href={`/${params.username}/${params.serviceName}`}
                          >
                            {params.serviceName}
                          </Link>
                        </>
                      }
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-5">
                  <div className="hidden min-[416px]:contents">
                    <div className="text-sm">
                      Connected to: {infoQuery.data.name}; Version: {infoQuery.data.version}
                    </div>
                    <Link target="_blank" href={process.env.NEXT_PUBLIC_DISCORD_INVITE} className={clsx(
                      "rounded-md px-2.5 py-1.5 text-sm",
                      "text-gray-900 bg-white hover:bg-gray-50 ring-1 ring-gray-200",
                      "dark:text-white",
                      "dark:bg-neutral-900 dark:hover:bg-white/10 dark:ring-white/20"
                    )}>
                      Join us on Discord
                    </Link>
                  </div>
                  <div className="hidden md:block md:h-5 md:w-px md:bg-zinc-900/10 md:dark:bg-white/15"/>
                  <ProfileMenu/>
                </div>
              </div>
              <div className="flex space-x-4">
                {
                  currentNavigation().map((item) => {
                    const current = isCurrentPath(pathname.toLowerCase(), item.href.toLowerCase());
                    return (
                      <Link
                        href={item.href}
                        target={item.href.startsWith("http") ? "_blank" : undefined}
                        key={item.name}
                        className={clsx(
                          "border-b-2",
                          current ?
                            "border-blue-500 dark:border-white" :
                            "border-transparent"
                        )}
                      >
                        <button className={clsx(
                          current ? "font-medium text-gray-900 dark:text-white" : "text-gray-600 dark:text-white/90",
                          "hover:bg-gray-100 px-2.5 py-1.5 text-sm rounded-md mb-1.5 inline-flex hover:text-gray-900",
                          "dark:hover:bg-neutral-900 dark:text-white dark:hover:text-neutral-100"
                        )}>
                          <item.icon className="-ml-1 mr-2 h-5 w-5" aria-hidden="true"/>
                          {item.name}
                        </button>
                      </Link>
                    );
                  })
                }
              </div>
            </div>
          </motion.nav>
          <main className="flex-1 relative overflow-y-auto focus:outline-none">
            <div className="py-6">
              <div className="max-w-7xl mx-auto lg:px-8 mt-24">
                <div className="flex flex-col">
                  <div className="overflow-x-auto sm:-mx-6 lg:-mx-8">
                    {children}
                  </div>
                </div>
              </div>
            </div>
          </main>
        </main>
      </div>
    </>
  )
};
