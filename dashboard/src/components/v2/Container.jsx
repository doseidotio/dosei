import React from "react";

const ContainerHeader = ({ title, children }) => {
  return (
    <div className="flex justify-between">
      <h2 className="text-3xl font-medium text-gray-900 dark:text-white mb-8 flex items-center">
        {title}
      </h2>
      <div className="ml-4 flex-shrink-0 space-x-4">
        {children}
      </div>
    </div>
  );
};

const ContainerBody = ({ children }) => {
  return children;
};

const Container = ({ children }) => {
  const header = React.Children.toArray(children).find(child => child.type === ContainerHeader);
  const body = React.Children.toArray(children).find(child => child.type === ContainerBody);
  return (
    <div className="flex flex-col">
      <div className="overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          {header}
          <div className="space-y-6 lg:px-0 lg:col-span-9">
            {body}
          </div>
        </div>
      </div>
    </div>
  )
};

Container.Header = ContainerHeader;
Container.Body = ContainerBody;

export default Container;
