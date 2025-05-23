import moment from "moment";
import {ChevronRightIcon} from "@heroicons/react/20/solid";

export const DeploymentRow = ({ deployment }) => {
  return (
    <li>
      <div className="block hover:bg-gray-50 dark:bg-neutral-900 dark:hover:bg-white/10">
        <div className="flex items-center px-4 py-4 sm:px-6">
          <div className="flex min-w-0 flex-1 items-center">
            <div className="min-w-0 flex-1 grid-col-2 md:grid md:grid-cols-3 md:gap-4">
              <div>
                <p className="text-sm text-gray-900 dark:text-white">
                  {deployment.id}
                </p>
                <p className="mt-2 flex items-center text-sm text-gray-500">
                  Last Accessed {moment(deployment.last_accessed_at).calendar()}
                </p>
              </div>
              <div>
                <div className="flex gap-x-1 items-center text-gray-500 dark:text-white/60 text-sm leading-6">
                  <dt>
                    <span className="text-gray-600">LOL</span> on <span className="text-gray-600">
                    {deployment.host_port}:{deployment.container_port}
                  </span>
                  </dt>
                  <svg fill="currentColor" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" className="h-5 mr-2">
                    <path d="M416,160a64,64,0,1,0-96.27,55.24c-2.29,29.08-20.08,37-75,48.42-17.76,3.68-35.93,7.45-52.71,13.93V151.39a64,64,0,1,0-64,0V360.61a64,64,0,1,0,64.42.24c2.39-18,16-24.33,65.26-34.52,27.43-5.67,55.78-11.54,79.78-26.95,29-18.58,44.53-46.78,46.36-83.89A64,64,0,0,0,416,160ZM160,64a32,32,0,1,1-32,32A32,32,0,0,1,160,64Zm0,384a32,32,0,1,1,32-32A32,32,0,0,1,160,448ZM352,192a32,32,0,1,1,32-32A32,32,0,0,1,352,192Z"/>
                  </svg>
                </div>
                <div className="flex gap-x-1 items-center text-gray-500 dark:text-white/60">
                  <dt className="text-sm leading-6">
                    Last updated <span className="text-gray-600 dark:text-white/40">{moment(deployment.updated_at).fromNow()}</span> via
                  </dt>
                  <svg fill="currentColor" viewBox="0 0 24 24" className="h-5 mr-2" aria-hidden="true">
                    <path
                      fillRule="evenodd"
                      d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                      clipRule="evenodd"
                    />
                  </svg>
                </div>
              </div>
              <div className="flex gap-x-1 items-center text-gray-500 text-sm">
                {/*{moment(deployment.updated_at).fromNow()} by <span className="text-gray-600">{deployment.git.commit.author.name}</span>*/}
                {/*{*/}
                {/*  deployment.git.commit.author.name &&*/}
                {/*  <img*/}
                {/*    className="ml-1 h-5 w-5 border rounded-full"*/}
                {/*    src={`https://github.com/${deployment.git.commit.author.name}.png`}*/}
                {/*    alt={`${deployment.git.commit.author.name} profile photo`}*/}

                {/*    onError={(e) => { e.target.onerror = null; e.target.src='/profile.png'; }}*/}
                {/*  />*/}
                {/*}*/}
              </div>
            </div>
          </div>
          <div>
            <ChevronRightIcon className="h-5 w-5 text-gray-400 dark:text-white/60" aria-hidden="true" />
          </div>
        </div>
      </div>
    </li>
  )
};
