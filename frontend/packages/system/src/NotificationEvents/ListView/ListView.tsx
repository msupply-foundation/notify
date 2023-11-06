import React from 'react';
import { useTranslation } from '@common/intl';
import {
  AppBarButtonsPortal,
  AppBarContentPortal,
  DataTable,
  NothingHere,
  RelativeTimeDate,
  TableProvider,
  Tooltip,
  Typography,
  createTableStore,
  useColumns,
} from '@common/ui';
import { useQueryParamsState } from '@common/hooks';
import { NotificationEventRowFragment, useNotificationEvents } from '../api';

import { ConfigKind, StringUtils, useNavigate } from '@notify-frontend/common';
import { FilterBar } from './FilterBar';

type ListViewProps = {
  kind: ConfigKind | null;
};

export const ListView = ({}: ListViewProps) => {
  const t = useTranslation('system');
  const navigate = useNavigate();

  const { filter, queryParams, updatePaginationQuery, updateSortQuery } =
    useQueryParamsState({
      initialSort: {
        key: 'createdAt',
        dir: 'desc',
      },
    });

  const columns = useColumns<NotificationEventRowFragment>(
    [
      { key: 'title', label: 'label.title' },
      { key: 'toAddress', label: 'label.address' },
      {
        key: 'message',
        label: 'label.message',
        sortable: false,
        Cell: props => (
          <Tooltip title={props.rowData.message}>
            <Typography>
              {StringUtils.ellipsis(props.rowData.message, 10)}
            </Typography>
          </Tooltip>
        ),
      },
      {
        key: 'createdAt',
        label: 'label.date',
        Cell: props => (
          <Tooltip title={props.rowData.createdAt}>
            <RelativeTimeDate d={props.rowData.createdAt}></RelativeTimeDate>
          </Tooltip>
        ),
      },
      {
        key: 'kind',
        label: 'label.kind',
        sortable: false,
        Cell: props => (
          <Typography>{props.rowData.notificationType}</Typography>
        ),
      },
      {
        key: 'status',
        label: 'label.status',
        sortable: true,
        Cell: props => <Typography>{props.rowData.status}</Typography>,
      },
      {
        key: 'errorMessage',
        label: 'error',
        sortable: true,
        Cell: props => (
          <Tooltip title={props.rowData.errorMessage ?? 'No Error Recorded'}>
            <Typography>
              {StringUtils.ellipsis(props.rowData.errorMessage ?? '', 10)}
            </Typography>
          </Tooltip>
        ),
      },
    ],
    { sortBy: queryParams.sortBy, onChangeSortBy: updateSortQuery },
    [queryParams.sortBy, updateSortQuery]
  );

  const { data, isError, isLoading } = useNotificationEvents(queryParams);
  const notificationEvents = data?.nodes ?? [];

  const pagination = {
    page: queryParams.page,
    offset: queryParams.offset,
    first: queryParams.first,
  };

  return (
    <>
      <AppBarButtonsPortal></AppBarButtonsPortal>
      <TableProvider createStore={createTableStore}>
        <AppBarContentPortal sx={{ paddingBottom: '16px', flex: 1 }}>
          <FilterBar filter={filter} />
        </AppBarContentPortal>
        <DataTable
          columns={columns}
          data={notificationEvents}
          isError={isError}
          isLoading={isLoading}
          onRowClick={evt => navigate(evt.id)}
          noDataElement={
            <NothingHere body={t('messages.no-events-matching-status')} />
          }
          pagination={pagination}
          onChangePage={updatePaginationQuery}
        />
      </TableProvider>
    </>
  );
};
