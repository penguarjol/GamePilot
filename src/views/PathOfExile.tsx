import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { CurrencyPrice } from "@/types";

const LEAGUES = ["Standard", "Settlers of Kalguur", "Hardcore", "SSF Standard"];

type SortKey = "name" | "chaos_equivalent" | "change_percent";
type SortDir = "asc" | "desc";

export function PathOfExile() {
  const [league, setLeague] = useState(LEAGUES[0]);
  const [prices, setPrices] = useState<CurrencyPrice[]>([]);
  const [loading, setLoading] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("chaos_equivalent");
  const [sortDir, setSortDir] = useState<SortDir>("desc");

  useEffect(() => {
    fetchPrices();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [league]);

  const fetchPrices = async () => {
    setLoading(true);
    try {
      const data = await invoke<CurrencyPrice[]>("get_poe_currency_prices", {
        league,
      });
      setPrices(data);
    } catch (err) {
      toast.error(`Failed to fetch prices: ${err}`);
      setPrices([]);
    } finally {
      setLoading(false);
    }
  };

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const sortedPrices = [...prices].sort((a, b) => {
    const mul = sortDir === "asc" ? 1 : -1;
    if (sortKey === "name") return mul * a.name.localeCompare(b.name);
    return mul * (a[sortKey] - b[sortKey]);
  });

  const sortIndicator = (key: SortKey) => {
    if (sortKey !== key) return "";
    return sortDir === "asc" ? " \u2191" : " \u2193";
  };

  const changeColor = (pct: number) =>
    pct > 0
      ? "text-emerald-400"
      : pct < 0
        ? "text-red-400"
        : "text-muted-foreground";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Path of Exile</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Currency exchange rates from poe.ninja
          </p>
        </div>
        <Button variant="outline" onClick={fetchPrices} disabled={loading}>
          {loading ? "Loading..." : "Refresh"}
        </Button>
      </div>

      {/* League Selector */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-base">League</CardTitle>
          <Badge variant="outline">{prices.length} currencies</Badge>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {LEAGUES.map((l) => (
              <Button
                key={l}
                variant={league === l ? "default" : "outline"}
                size="sm"
                onClick={() => setLeague(l)}
              >
                {l}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Currency Table */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Currency Exchange Rates</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <p className="text-sm text-muted-foreground py-8 text-center">
              Loading currency data...
            </p>
          ) : sortedPrices.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead
                    className="cursor-pointer select-none"
                    onClick={() => toggleSort("name")}
                  >
                    Currency{sortIndicator("name")}
                  </TableHead>
                  <TableHead
                    className="text-right cursor-pointer select-none"
                    onClick={() => toggleSort("chaos_equivalent")}
                  >
                    Chaos Value{sortIndicator("chaos_equivalent")}
                  </TableHead>
                  <TableHead
                    className="text-right cursor-pointer select-none"
                    onClick={() => toggleSort("change_percent")}
                  >
                    24h Change{sortIndicator("change_percent")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedPrices.map((c) => (
                  <TableRow key={c.name}>
                    <TableCell className="font-medium">{c.name}</TableCell>
                    <TableCell className="text-right font-mono">
                      {c.chaos_equivalent.toFixed(2)}c
                    </TableCell>
                    <TableCell
                      className={`text-right font-mono ${changeColor(c.change_percent)}`}
                    >
                      {c.change_percent > 0 ? "+" : ""}
                      {c.change_percent.toFixed(1)}%
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm text-muted-foreground py-8 text-center">
              No currency data available for this league
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
