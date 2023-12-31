import React, { FC, useEffect } from 'react';

import {
  AppFooterPortal,
  Box,
  SnackbarProvider,
  AppFooter,
  Routes,
  Route,
  RouteBuilder,
  useLocation,
  useHostContext,
  useGetPageTitle,
  DetailPanel,
  Navigate,
} from '@notify-frontend/common';
import { AppDrawer, AppBar, Footer, NotFound } from './components';
import { CommandK } from './CommandK';
import { AppRoute } from '@notify-frontend/config';
import { Settings } from './Admin/Settings';
import { UserAccountRouter } from './routers';
import { RequireAuthentication } from './components/Navigation/RequireAuthentication';
import { QueryErrorHandler } from './QueryErrorHandler';
import { RecipientsRouter } from './routers/RecipientsRouter';
import { NotificationsRouter } from './routers/NotificationsRouter';
import { NotificationEventsRouter } from './routers/NotificationEventsRouter';
import { QueriesRouter } from './routers/QueryRouter';

export const Site: FC = () => {
  const location = useLocation();
  const getPageTitle = useGetPageTitle();
  const { setPageTitle } = useHostContext();

  useEffect(() => {
    setPageTitle(getPageTitle(location.pathname));
  }, [location]);

  return (
    <RequireAuthentication>
      <CommandK>
        <SnackbarProvider maxSnack={3}>
          <AppDrawer />
          <Box flex={1} display="flex" flexDirection="column" overflow="hidden">
            <AppBar />
            <Box display="flex" flex={1} overflow="auto">
              <Routes>
                <Route
                  path={RouteBuilder.create(AppRoute.UserAccounts)
                    .addWildCard()
                    .build()}
                  element={<UserAccountRouter />}
                />
                <Route
                  path={RouteBuilder.create(AppRoute.Admin)
                    .addWildCard()
                    .build()}
                  element={<Settings />}
                />
                <Route
                  path={RouteBuilder.create(AppRoute.Recipients)
                    .addWildCard()
                    .build()}
                  element={<RecipientsRouter />}
                />{' '}
                <Route
                  path={RouteBuilder.create(AppRoute.Notifications)
                    .addWildCard()
                    .build()}
                  element={<NotificationsRouter />}
                />
                <Route
                  path={RouteBuilder.create(AppRoute.NotificationEvents)
                    .addWildCard()
                    .build()}
                  element={<NotificationEventsRouter />}
                />
                <Route
                  path={RouteBuilder.create(AppRoute.Queries)
                    .addWildCard()
                    .build()}
                  element={<QueriesRouter />}
                />
                <Route
                  path="/"
                  element={
                    <Navigate
                      replace
                      to={RouteBuilder.create(AppRoute.Notifications).build()}
                    />
                  }
                />
                <Route path="*" element={<NotFound />} />
              </Routes>
            </Box>
            <AppFooter />
            <AppFooterPortal SessionDetails={<Footer />} />
          </Box>
          <DetailPanel />
          <QueryErrorHandler />
        </SnackbarProvider>
      </CommandK>
    </RequireAuthentication>
  );
};
