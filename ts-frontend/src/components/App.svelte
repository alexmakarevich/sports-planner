<script lang="ts">
  import Home from "./Home.svelte";
  import { Router, Route, navigate } from "svelte-routing";
  import LoggedInFrame from "./LoggedInFrame.svelte";
  import Login from "./Login.svelte";
  import NotFound from "./NotFound.svelte";
  import Users from "./users/Users.svelte";
  import Events from "./events/Events.svelte";
  import CreateEvent from "./events/CreateEvent.svelte";
  import NotificationWrapper from "./global/NotificationWrapper.svelte";
  import ListTeams from "./teams/ListTeams.svelte";

  const queryParams = new URLSearchParams(window.location.search);
  const redirect = queryParams.get("fe-route");
  if (redirect) {
    navigate(redirect, { replace: true });
  }
</script>

<NotificationWrapper />
<Router basepath="/">
  <!-- FYI: this router mess is due to the way svelte-routing handles fallbacks and generic routes w/ conditional HTML elements -->
  <Route path="login"><Login /></Route>

  <Route path="/*">
    <Router>
      <Route path="/*">
        <!--  -->
        <LoggedInFrame>
          <Router>
            <Route path="/">
              <Home />
            </Route>
            <Route path="/users">
              <Users />
            </Route>
            <Route path="/teams">
              <ListTeams />
            </Route>
            <Route path="users">
              <Users />
            </Route>
            <Route path="events">
              <Events />
            </Route>
            <Route path="create-event">
              <CreateEvent />
            </Route>
          </Router>
        </LoggedInFrame>
        <!--  -->
      </Route>
      <Route>
        <NotFound />
      </Route>
    </Router>
  </Route>
</Router>
<footer><section>some footer stuff © 2077</section></footer>

<style>
</style>
