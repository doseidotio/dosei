'use client'

import ConsoleLayout from "@/app/ConsoleLayout";
import { useParams } from "next/navigation";
import {useQuery} from "@tanstack/react-query";
import {DeploymentRow} from "@/components/DeploymentRow";
import {api} from "@/lib/api";
import {ProjectRowSkeleton} from "@/app/[username]/page";

export default function ServicePage() {
  const params = useParams();
  const serviceDeploymentsQuery = useQuery({
    queryKey: [`service/${params.serviceName}/deployment`],
    queryFn: async () => {
      const response = await api.get(`/service/${params.serviceName}/deployment`);
      return await response.json();
    },
  });

  return (
    <ConsoleLayout>
      <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
        <div className="flex justify-between">
          <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
            {params.serviceName}
          </h2>
        </div>
        <div className="space-y-6 lg:px-0 lg:col-span-9">
          <div className="overflow-hidden border rounded-md border-neutral-800">
            <div className="overflow-hidden bg-white dark:bg-neutral-900">
              <ul role="list" className="divide-y divide-gray-200 dark:divide-neutral-800">
                {
                  (serviceDeploymentsQuery.isLoading) ? [...Array(3).keys()].map(key => <ProjectRowSkeleton key={key}/>) :
                    serviceDeploymentsQuery.data.map(deployment => <DeploymentRow key={deployment.id} deployment={deployment}/>)
                }
              </ul>
            </div>
          </div>
        </div>
      </div>
    </ConsoleLayout>
  );
}
