<script lang="js">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition'

    export let message;
    export let delay;

    const dispatch = createEventDispatcher();

    const removeAlert = (id) => {
        dispatch('change', {
            id
        })
    }

    onMount(() => {
        setTimeout(() => removeAlert(message.id), delay);
    })

</script>

<div class="uk-alert-danger" out:fade>
    {message.text}
</div>

<style lang="stylus">
    .uk-alert-danger
        margin: 5px 0 10px 0;
        padding: 10px 16px;
</style>