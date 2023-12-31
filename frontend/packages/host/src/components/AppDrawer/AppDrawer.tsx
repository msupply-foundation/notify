import React from 'react';
import { Property } from 'csstype';
import {
  Box,
  Divider,
  List,
  SettingsIcon,
  Theme,
  IconButton,
  styled,
  useDrawer,
  useTranslation,
  AppNavLink,
  useIsMediumScreen,
  useAuthContext,
  useLocation,
  EnvUtils,
  UsersIcon,
  ExternalNavLink,
  BookIcon,
  LogoutIcon,
  SuppliersIcon,
  RouteBuilder,
  SidebarIcon,
  MessagesIcon,
  PersonSearchIcon,
  SearchIcon,
  ListIcon,
} from '@notify-frontend/common';
import { AppRoute, ExternalURL } from '@notify-frontend/config';
import { AppDrawerIcon } from './AppDrawerIcon';

const ToolbarIconContainer = styled(Box)({
  display: 'flex',
  height: 90,
  justifyContent: 'center',
});

const commonListContainerStyles = {
  alignItems: 'center',
  display: 'flex',
  flexDirection: 'column' as Property.FlexDirection,
};

const LowerListContainer = styled(Box)({
  ...commonListContainerStyles,
});

const UpperListContainer = styled(Box)({
  ...commonListContainerStyles,
  flex: 1,
  msOverflowStyle: 'none',
  overflow: 'scroll',
  scrollbarWidth: 'none',
  '&::-webkit-scrollbar': {
    display: 'none',
  },
});

const StyledDivider = styled(Divider)({
  marginLeft: 8,
  width: 152,
});

const drawerWidth = 240;

const getDrawerCommonStyles = (theme: Theme) => ({
  backgroundColor: theme.palette.background.drawer,
  overflow: 'hidden',
});

const openedMixin = (theme: Theme) => ({
  ...getDrawerCommonStyles(theme),
  width: drawerWidth,

  transition: theme.transitions.create('width', {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.enteringScreen,
  }),
});

const closedMixin = (theme: Theme) => ({
  ...getDrawerCommonStyles(theme),

  transition: theme.transitions.create('width', {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.leavingScreen,
  }),
  [theme.breakpoints.up('sm')]: {
    width: theme.spacing(10),
  },
});

const StyledDrawer = styled(Box, {
  shouldForwardProp: prop => prop !== 'isOpen',
})<{ isOpen: boolean }>(({ isOpen, theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  height: '100%',
  borderRadius: '0 8px 8px 0',
  overflow: 'hidden',
  boxShadow: theme.shadows[7],
  zIndex: theme.zIndex.drawer,
  '& .MuiSvgIcon-root': {
    color: theme.mixins.drawer?.iconColor,
  },
  '& .navLinkText .MuiTypography-root': {
    color: theme.mixins.drawer?.textColor,
  },
  ...(isOpen && {
    ...openedMixin(theme),
    '& .MuiDrawer-paper': openedMixin(theme),
    '& .navLinkText': {
      display: 'inline-flex',
      flex: 1,
    },
    '& div > ul > li': {
      width: 200,
    },
  }),
  ...(!isOpen && {
    ...closedMixin(theme),
    '& .MuiDrawer-paper': closedMixin(theme),
    '& .navLinkText': {
      display: 'none',
    },
    '& div > ul > li': {
      width: 40,
    },
  }),
}));

export const AppDrawer: React.FC = () => {
  const t = useTranslation(['host']);
  const isMediumScreen = useIsMediumScreen();
  const drawer = useDrawer();
  const { logout } = useAuthContext();
  const location = useLocation();

  React.useEffect(() => {
    if (drawer.hasUserSet) return;
    if (isMediumScreen && drawer.isOpen) drawer.close();
    if (!isMediumScreen && !drawer.isOpen) drawer.open();
  }, [isMediumScreen]);

  const onHoverOut = () => {
    if (!drawer.hoverOpen) return;

    drawer.close();
    drawer.setHoverOpen(false);
  };

  const onHoverOver = () => {
    if (drawer.isOpen) return;

    drawer.open();
    drawer.setHoverOpen(true);
  };

  const docsUrl = `${ExternalURL.PublicDocs}${
    EnvUtils.mapRoute(location.pathname).docs
  }`;

  return (
    <StyledDrawer
      data-testid="drawer"
      aria-expanded={drawer.isOpen}
      isOpen={drawer.isOpen}
    >
      <ToolbarIconContainer>
        <IconButton
          label={t(
            drawer.isOpen ? 'button.close-the-menu' : 'button.open-the-menu'
          )}
          onClick={drawer.toggle}
          icon={<AppDrawerIcon />}
          sx={{ '&:hover': { backgroundColor: 'inherit' } }}
        />
      </ToolbarIconContainer>
      <div
        style={{
          display: 'flex',
          flex: 1,
          flexDirection: 'column',
        }}
        onMouseEnter={onHoverOver}
        onMouseLeave={onHoverOut}
      >
        <UpperListContainer>
          <List>
            <AppNavLink
              to={AppRoute.Notifications}
              icon={<MessagesIcon fontSize="small" color="primary" />}
              text={t('notifications')}
            />
            <AppNavLink
              to={AppRoute.NotificationEvents}
              icon={<ListIcon fontSize="small" color="primary" />}
              text={t('notification-events')}
            />
            <AppNavLink
              to={AppRoute.Recipients}
              icon={<SuppliersIcon fontSize="small" color="primary" />}
              text={t('recipients')}
            />
            <AppNavLink
              to={RouteBuilder.create(AppRoute.Recipients)
                .addPart(AppRoute.RecipientLists)
                .build()}
              icon={<SidebarIcon fontSize="small" color="primary" />}
              text={t('recipient-lists')}
            />
            <AppNavLink
              to={RouteBuilder.create(AppRoute.Recipients)
                .addPart(AppRoute.SqlRecipientLists)
                .build()}
              icon={<PersonSearchIcon fontSize="small" color="primary" />}
              text={t('sql-recipient-lists')}
            />
            <AppNavLink
              to={RouteBuilder.create(AppRoute.Queries).build()}
              icon={<SearchIcon fontSize="small" color="primary" />}
              text={t('label.queries')}
            />
            <AppNavLink
              to={AppRoute.UserAccounts}
              icon={<UsersIcon fontSize="small" color="primary" />}
              text={t('users')}
            />
          </List>
        </UpperListContainer>
        <LowerListContainer>
          <List>
            {drawer.isOpen && <StyledDivider color="drawerDivider" />}
            <ExternalNavLink
              to={docsUrl}
              icon={<BookIcon fontSize="small" color="primary" />}
              text={t('docs')}
              trustedSite={true}
            />
            <AppNavLink
              to={AppRoute.Admin}
              icon={<SettingsIcon fontSize="small" color="primary" />}
              text={t('admin')}
            />
            <AppNavLink
              to={AppRoute.Login}
              icon={<LogoutIcon fontSize="small" color="primary" />}
              text={t('logout')}
              onClick={logout}
            />
          </List>
        </LowerListContainer>
      </div>
    </StyledDrawer>
  );
};

export default AppDrawer;
