<script lang="js">
  import { createGraphQlClient } from '../utils/util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import { Statistics } from '../../api/dashboard.graphql';
  import { statusesList } from '../constants/statuses';

  const gqlClient = createGraphQlClient(
    '/api-dashboard',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const dashboard = subscribe(Statistics, { errorPolicy: 'all' });

  $: error = $dashboard && $dashboard.error;
  $: isLoading = !isOnline || $dashboard.loading;
  $: canRenderMainComponent = isOnline && $dashboard.data;
  $: json_data = JSON.stringify($dashboard.data);
  $: stat = $dashboard.data && $dashboard.data.statistics;

  function getStatusCount(items, status) {
    const output = items.find((x) => x.status === status);
    return output ? output.count : 0;
  }
</script>

<template>
  <Shell {canRenderMainComponent} {isLoading} {error}>
    <div slot="main" class="main">
      <section
        class="uk-section uk-section-muted single-output"
        style="visibility: visible"
      >
        <div>{json_data}</div>
      </section>
      {#each stat as client}
        <section class="uk-section-muted toolbar">
          <span class="section-label">{client.id}</span>

          {#if client.statistics.data}
            <div class="uk-grid uk-grid-small">
              <div class="uk-width-1-2@m uk-width-1-3@s">
                <span class="toolbar-label total-inputs-label">
                  INPUTS:
                  {#each statusesList as status (status)}
                    <div
                      class="status"
                      class:online={status === 'ONLINE'}
                      class:offline={status === 'OFFLINE'}
                      class:initializing={status === 'INITIALIZING'}
                      class:unstable={status === 'UNSTABLE'}
                    >
                      {getStatusCount(client.statistics.data.inputs, status)}
                    </div>
                  {/each}
                </span>
              </div>

              <div class="uk-width-expand">
                <span class="toolbar-label">
                  OUTPUTS:
                  {#each statusesList as status (status)}
                    <div
                      class="status"
                      class:online={status === 'ONLINE'}
                      class:offline={status === 'OFFLINE'}
                      class:initializing={status === 'INITIALIZING'}
                      class:unstable={status === 'UNSTABLE'}
                    >
                      {getStatusCount(client.statistics.data.outputs, status)}
                    </div>
                  {/each}
                </span>
              </div>
            </div>
          {:else}
            <span>
              {client.statistics.errors}
            </span>
          {/if}
        </section>
      {/each}
    </div>
  </Shell>
</template>

<style lang="stylus">
  .status
    min-width: 28px
    display: inline-flex
    padding-left: 4px

</style>
