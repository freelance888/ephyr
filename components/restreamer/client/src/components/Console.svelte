<script lang="js">
  import { mutation, subscribe } from 'svelte-apollo';
  import { ConsoleLog } from '../../api/dashboard.graphql';
  import { ConsoleClear } from '../../api/dashboard.graphql';
  import { createEventDispatcher } from 'svelte';
  import { showError } from '../utils/util';

  const consoleLog = subscribe(ConsoleLog, { errorPolicy: 'all' });
  const consoleClear = mutation(ConsoleClear, { errorPolicy: 'all' });

  const dispatch = createEventDispatcher();

  $: items = $consoleLog.data?.consoleLog ?? [];

  $: errorCount = items.filter((x) => x.kind === 'ERR').length;
  $: warningCount = items.filter((x) => x.kind === 'WARNING').length;
  $: infoCount = items.filter((x) => x.kind === 'INFO').length;

  let isOpen = false;
  const dispatchToggleConsole = () => {
    isOpen = !isOpen;
    dispatch('toggleConsole', isOpen);
  };

  const clearConsole = async () => {
    try {
      await consoleClear();
    } catch (e) {
      showError(e.message);
    }
  };
</script>

<template>
  <section>
    <div class="console-toolbar uk-flex uk-flex-middle">
      <span class="console-title" on:click={dispatchToggleConsole}>Console</span
      >
      {#if errorCount}
        <span class="console-icon" title="Number of errors">
          <i class="fa fa-exclamation-triangle console-error" />{errorCount}
        </span>
      {/if}

      {#if warningCount}
        <span class="console-icon" title="Number of warning messages">
          <i class="fa fa-exclamation-triangle console-warning" />{warningCount}
        </span>
      {/if}

      {#if infoCount}
        <span class="console-icon" title="Number of info messages">
          <i class="fa fa-exclamation-triangle" />{infoCount}
        </span>
      {/if}

      {#if items.length}
        <a
          href="/"
          class="uk-display-inline-block uk-margin-left"
          on:click={clearConsole}>Clear ({items.length})</a
        >
      {/if}
    </div>

    <ul class="messages-container uk-list uk-list-divider uk-list-collapse">
      {#each items as item}
        <li
          class:uk-text-danger={item.kind === 'ERR'}
          class:uk-text-warning={item.kind === 'WARNING'}
        >
          <span class="source">
            {item.source}
          </span>
          <span>
            {item.message}
          </span>
        </li>
      {/each}
    </ul>
  </section>
</template>

<style lang="stylus">

  section
    height: 100%
    padding: 0 16px 16px 16px
    font-size: smaller

  .messages-container
    height: calc(100% - 50px)
    margin-top: 8px
    overflow-y: auto


  .console-title
    padding: 6px 0
    cursor: pointer
    font-weight: bold
    flex: 1

  .source
    padding: 4px
    background: #999
    color: #fff
    border-radius: 4px
    font-size: smaller
    font-weight: bold

  .console-icon
    padding: 8px

  .fa-exclamation-triangle
    display: inline-block
    margin-right: 4px

  .console-error
    color: var(--danger-color)
  .console-warning
    color: var(--warning-color)

</style>
