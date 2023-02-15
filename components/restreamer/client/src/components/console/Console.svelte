<script lang="js">
import { subscribe } from 'svelte-apollo';
import { ConsoleLog } from '../../../api/dashboard.graphql';

  const consoleLog = subscribe(ConsoleLog, { errorPolicy: 'all' });

  import { createEventDispatcher } from 'svelte'
  const dispatch = createEventDispatcher()

  $: items = $consoleLog.data?.consoleLog ?? [];
  $: {
    console.log(items);
  }


  let isOpen = false;
  const dispatchToggleConsole = () => {
    isOpen = !isOpen;
    dispatch('toggleConsole', isOpen)
  };

</script>

<template>
 <section>
  <div class='console-toolbar uk-flex' on:click={dispatchToggleConsole}>
    <span class='console-title'>Console</span>
    {#if items.length}
      <a class='clear-btn uk-margin-auto-left'>Clear ({items.length})</a>
    {/if}
  </div>

  <ul class="messages-container uk-list uk-list-divider uk-list-collapse">
    {#each items as item}
     <li
       class:uk-text-danger={item.kind === 'ERR'}
       class:uk-text-warning={item.kind === 'WARNING'}
     >
       <span class='source'>
         { item.source }
       </span>
       <span>
        { item.message }
       </span>
     </li>
    {/each}
  </ul>
 </section>
</template>

<style lang="stylus">

 section
   padding: 0 16px 16px 16px
   font-size: 15px

 .console-toolbar
  padding: 6px 0
  cursor: pointer

 .messages-container
  margin-top: 8px;

 .console-title
  font-weight: bold

 .source
   padding: 4px
   background: #999
   color: #fff
   border-radius: 4px
   font-size: smaller
   font-weight: bold


</style>
