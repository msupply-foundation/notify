import React, { FC } from 'react';
import { RouteBuilder, Navigate, useMatch } from '@notify-frontend/common';
import { AppRoute } from '@notify-frontend/config';

const UserAccountService = React.lazy(
  () => import('@notify-frontend/system/src/Users/Service/Service')
);

const fullUserAccountsPath = RouteBuilder.create(AppRoute.UserAccounts)
  .addWildCard()
  .build();

export const UserAccountRouter: FC = () => {
  const gotoUserAccounts = useMatch(fullUserAccountsPath);

  if (gotoUserAccounts) {
    return <UserAccountService />;
  }

  const notFoundRoute = RouteBuilder.create(AppRoute.PageNotFound).build();
  return <Navigate to={notFoundRoute} />;
};
