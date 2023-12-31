import { useTranslation } from '@common/intl';
import {
  AppBarButtonsPortal,
  AppBarContentPortal,
  DataTable,
  LoadingButton,
  NothingHere,
  PlusCircleIcon,
  SearchAndDeleteToolbar,
  TableProvider,
  createTableStore,
  useColumns,
} from '@common/ui';
import { useNavigate, useQueryParamsState } from 'packages/common/src';
import React from 'react';
import { useDeleteSqlRecipientList, useSqlRecipientLists } from '../api';
import { SqlRecipientListRowFragment } from '../api/operations.generated';

export const AllSqlLists = () => {
  const t = useTranslation('system');
  const navigate = useNavigate();

  const { filter, queryParams, updatePaginationQuery, updateSortQuery } =
    useQueryParamsState();

  const columns = useColumns<SqlRecipientListRowFragment>(
    [
      { key: 'name', label: 'label.name' },
      {
        key: 'description',
        label: 'label.description',
        maxWidth: 300,
        sortable: false,
      },
      {
        key: 'query',
        label: 'label.query',
        maxWidth: 300,
        sortable: false,
      },
      {
        key: 'parameters',
        label: 'label.parameters',
        maxWidth: 300,
        sortable: false,
      },
      'selection',
    ],
    {
      onChangeSortBy: updateSortQuery,
      sortBy: queryParams.sortBy,
    },
    [updateSortQuery, queryParams.sortBy]
  );

  const { mutateAsync: deleteSqlRecipientList, invalidateQueries } =
    useDeleteSqlRecipientList();

  const { data, isError, isLoading } = useSqlRecipientLists(queryParams);
  const recipientLists = data?.nodes ?? [];

  const pagination = {
    page: queryParams.page,
    offset: queryParams.offset,
    first: queryParams.first,
  };

  return (
    <>
      <AppBarButtonsPortal>
        <LoadingButton
          isLoading={false}
          startIcon={<PlusCircleIcon />}
          onClick={() => navigate('new')}
        >
          {t('label.new-sql-recipient-list')}
        </LoadingButton>
      </AppBarButtonsPortal>

      <TableProvider createStore={createTableStore}>
        <AppBarContentPortal sx={{ paddingBottom: '16px', flex: 1 }}>
          <SearchAndDeleteToolbar
            data={recipientLists}
            filter={filter}
            deleteItem={deleteSqlRecipientList}
            invalidateQueries={invalidateQueries}
          />
        </AppBarContentPortal>
        <DataTable
          pagination={{ ...pagination, total: data?.totalCount }}
          onChangePage={updatePaginationQuery}
          columns={columns}
          data={recipientLists}
          isError={isError}
          isLoading={isLoading}
          onRowClick={list => navigate(list.id)}
          noDataElement={<NothingHere body={t('error.no-recipient-lists')} />}
        />
      </TableProvider>
    </>
  );
};
