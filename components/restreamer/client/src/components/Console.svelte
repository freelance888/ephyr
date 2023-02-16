<script lang="js">
  import { mutation, subscribe } from 'svelte-apollo';
  import { ConsoleLog } from '../../api/dashboard.graphql';
  import { ConsoleClear } from '../../api/dashboard.graphql';

  const consoleLog = subscribe(ConsoleLog, { errorPolicy: 'all' });
  const consoleClear = mutation(ConsoleClear, { errorPolicy: 'all' });

  import { createEventDispatcher } from 'svelte';
  import { showError } from '../utils/util';

  const dispatch = createEventDispatcher();

  $: items = $consoleLog.data?.consoleLog ?? [];

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
      {#if items.length}
        <a class="clear-btn uk-margin-auto-left" on:click={clearConsole}
          >Clear ({items.length})</a
        >
      {/if}
    </div>

    <ul class="messages-container uk-list uk-list-divider">
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
    font-size: 15px

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

</style>
